import Lean
import Catherine
import Catherine.Mirror

/-!
# Purpose: the `lean-parity` oracle — enumerates the check-applicable fixture
corpus (`fixtures/{katas,parity,probes}/**/*.ct.json`), projects each
document to the trusted mirror, runs Lean `build` + `check`, runs Rust
`catherine check --json`, compares normalized finding multisets
(kind + label + witness), and reports machine-readable agreement or
divergence.

## Responsibilities: resolve the workspace root (`LEAN_PARITY_ROOT`, else
walk up for `Cargo.toml` + `crates/catherine-cli`) and the Rust binary
(`CATHERINE_BIN`, else `target/debug/catherine`); per document route on
the CLI exit code (0/3/4/5/6/7 → compare findings, 1 → `skipped_parse`
with the first stderr line, 2/spawn failure/other → harness error); for
`expected.json` `route: presentation` assert the parser rejects the
document and compare Lean findings against `expected.outcomes`; with
`--check-build` compare Lean build reports/tables to
`expected.{tables,dangling,typing_violations}`; optionally emit one
mirror JSON per projected fixture (`--emit-mirror`) and annotate
divergence attribution from `lean/parity-dispositions.json` (`side`,
`beads`) without ever suppressing a divergence.

## Rationale: one oracle, one comparison rule — normalized multisets, never
raw bytes or text output — so the twin's green always means "same
semantics". Harness-level failures (bad JSON shape, unresolvable row
names, spawn errors) exit 2 and never masquerade as semantic divergences
(design Decision 4): only a genuine findings mismatch exits 1.
-/

namespace Catherine
namespace Parity

open Lean

/-! ## Options -/

/-- The oracle's command-line options. -/
structure Options where
  emitMirror : Option String := none
  checkBuild : Bool := false
  corpus : Option String := none
  verbose : Bool := false
  report : Option String := none
  deriving Repr

/-- Parse the oracle's flags; anything unrecognized is a harness error. -/
def parseArgs : List String → Except String Options
  | [] => pure {}
  | "--check-build" :: rest => do
    let options ← parseArgs rest
    pure { options with checkBuild := true }
  | "--verbose" :: rest => do
    let options ← parseArgs rest
    pure { options with verbose := true }
  | "--emit-mirror" :: dir :: rest => do
    let options ← parseArgs rest
    pure { options with emitMirror := some dir }
  | "--corpus" :: dir :: rest => do
    let options ← parseArgs rest
    pure { options with corpus := some dir }
  | "--report" :: path :: rest => do
    let options ← parseArgs rest
    pure { options with report := some path }
  | flag :: _ => throw s!"unknown or incomplete flag `{flag}`"

/-! ## Report shapes -/

/-- One findings divergence: both normalized views plus the attribution. -/
structure Divergence where
  fixture : String
  rust : List String
  lean : List String
  side : String
  beads : Option String

/-- One build-report or table mismatch under `--check-build`. -/
structure BuildMismatch where
  fixture : String
  aspect : String
  expected : List String
  lean : List String

/-- The oracle's accumulated evidence. -/
structure Summary where
  total : Nat := 0
  rustCheckable : Nat := 0
  skipped : List (String × String) := []
  presentation : List String := []
  triviallyAgreeing : List String := []
  findingsBearingAgreeing : Nat := 0
  divergences : List Divergence := []
  buildMismatches : List BuildMismatch := []

/-- One document's contribution to the summary. -/
private structure DocOutcome where
  display : String
  checkable : Bool := false
  presentation : Bool := false
  skipped : Option (String × String) := none
  triviallyAgreeing : Option String := none
  findingsBearingAgreeing : Bool := false
  divergence : Option Divergence := none
  buildMismatches : List BuildMismatch := []

/-- A disposition entry from `lean/parity-dispositions.json`. -/
private structure Disposition where
  side : String
  beads : Option String

/-! ## Rendering and normalization -/

private def lessString (a b : String) : Bool :=
  compare a b != .gt

private def sortedFindings (findings : List String) : List String :=
  findings.mergeSort lessString

private def renderList (render : α → String) (items : List α) : String :=
  "[" ++ String.intercalate "," (items.map render) ++ "]"

private def renderNats (items : List Nat) : String :=
  renderList toString items

private def renderStrings (items : List String) : String :=
  renderList (fun item => item) items

private def renderOption : Option RowId → String
  | none => "none"
  | some row => s!"some({row})"

private def renderCell : Option RowId → String
  | none => "null"
  | some row => toString row

private def renderCells (cells : List (Option RowId)) : String :=
  renderList renderCell cells

private def renderTableEntry (entry : Ident × List (Option RowId)) : String :=
  s!"{entry.1}={renderCells entry.2}"

private def renderTables (tables : List (Ident × List (Option RowId))) : List String :=
  tables.map renderTableEntry

private def renderDanglingCell (cell : Instance.DanglingCell) : String :=
  s!"{cell.arrow}:{cell.src}"

private def renderTypingViolation (violation : Instance.TypingViolation) : String :=
  s!"{violation.arrow}:{violation.src}:{violation.value}"

/-- The canonical rendering of one Lean finding: kind + label + witness
fields. Rust findings are normalized into exactly this syntax. -/
def renderFinding (finding : Finding) : String :=
  let label := finding.label.asString
  match finding.kind with
  | .violation (.equation w) =>
    s!"violation|{label}|equation|equation={w.equation},row={w.row},lhs={w.lhs},rhs={w.rhs}"
  | .violation (.key w) =>
    s!"violation|{label}|key|key={w.key},rows={renderNats [w.rows.1, w.rows.2]}"
  | .violation (.naturality w) =>
    s!"violation|{label}|naturality|arrow={w.arrow},source_row={w.sourceRow},lhs={renderOption w.lhs},rhs={renderOption w.rhs}"
  | .violation (.coproduct w) =>
    s!"violation|{label}|coproduct|coproduct={w.coproduct},row={w.row},injections={renderStrings w.injections}"
  | .violation (.pullback (.commutes w)) =>
    s!"violation|{label}|pullback|commutes|pullback={w.pullback},row={w.row},lhs={w.lhs},rhs={w.rhs}"
  | .violation (.pullback (.covers w)) =>
    s!"violation|{label}|pullback|covers|pullback={w.pullback},pair={renderNats [w.pair.1, w.pair.2]}"
  | .violation (.pullback (.unique w)) =>
    s!"violation|{label}|pullback|unique|pullback={w.pullback},rows={renderNats [w.rows.1, w.rows.2]}"
  | .dangling w =>
    s!"dangling|{label}|dangling|arrow={w.arrow},src={w.src}"
  | .redundant w =>
    s!"redundant|{label}|redundant|derivable={renderStrings w.derivable},reducing={renderStrings w.reducing}"
  | .contradiction w =>
    s!"contradiction|{label}|contradiction|scope={renderNats w.scope}"
  | .underconstraint w =>
    match w.case with
    | .parallelPaths lhs rhs =>
      s!"underconstraint|{label}|underconstraint|case=parallel_paths,lhs={renderStrings lhs},rhs={renderStrings rhs},question={w.question}"
    | .uncoveredInputs cls examples =>
      s!"underconstraint|{label}|underconstraint|case=uncovered_inputs,class={cls},examples={renderNats examples},question={w.question}"

/-! ## Rust envelope normalization -/

private def fieldStr (node : Json) (key : String) : Except String String :=
  node.getObjVal? key >>= Json.getStr?

private def numField (node : Json) (key : String) : Except String Nat :=
  node.getObjVal? key >>= Json.getNat?

private def optionalNum (node : Json) (key : String) : Except String (Option Nat) :=
  match node.getObjValD key with
  | .null => pure none
  | value => do
    let row ← value.getNat?
    pure (some row)

private def numListField (node : Json) (key : String) : Except String (List Nat) := do
  let values ← node.getObjVal? key >>= Json.getArr?
  values.toList.mapM Json.getNat?

private def strListField (node : Json) (key : String) : Except String (List String) := do
  let values ← node.getObjVal? key >>= Json.getArr?
  values.toList.mapM Json.getStr?

private def optionalStrList (node : Json) (key : String) : Except String (List String) :=
  match node.getObjValD key with
  | .null => pure []
  | value => do
    let values ← value.getArr?
    values.toList.mapM Json.getStr?

/-- The violation witness's untagged variant inferred from its field set
(`equation` | `key` | `source_row` | `coproduct` | `pullback`). -/
private def normViolationWitness (label : String) (witness : Json) : Except String String := do
  let fields ← witness.getObj?
  if (fields.get? "equation").isSome then
    let equation ← numField witness "equation"
    let row ← numField witness "row"
    let lhs ← numField witness "lhs"
    let rhs ← numField witness "rhs"
    pure s!"violation|{label}|equation|equation={equation},row={row},lhs={lhs},rhs={rhs}"
  else if (fields.get? "key").isSome then
    let key ← numField witness "key"
    let rows ← numListField witness "rows"
    pure s!"violation|{label}|key|key={key},rows={renderNats rows}"
  else if (fields.get? "source_row").isSome then
    let arrow ← fieldStr witness "arrow"
    let sourceRow ← numField witness "source_row"
    let lhs ← optionalNum witness "lhs"
    let rhs ← optionalNum witness "rhs"
    pure s!"violation|{label}|naturality|arrow={arrow},source_row={sourceRow},lhs={renderOption lhs},rhs={renderOption rhs}"
  else if (fields.get? "coproduct").isSome then
    let coproduct ← numField witness "coproduct"
    let row ← numField witness "row"
    let injections ← optionalStrList witness "injections"
    pure s!"violation|{label}|coproduct|coproduct={coproduct},row={row},injections={renderStrings injections}"
  else if (fields.get? "pullback").isSome then
    let pullback ← numField witness "pullback"
    if (fields.get? "pair").isSome then
      let pair ← numListField witness "pair"
      pure s!"violation|{label}|pullback|covers|pullback={pullback},pair={renderNats pair}"
    else if (fields.get? "rows").isSome then
      let rows ← numListField witness "rows"
      pure s!"violation|{label}|pullback|unique|pullback={pullback},rows={renderNats rows}"
    else
      let row ← numField witness "row"
      let lhs ← numField witness "lhs"
      let rhs ← numField witness "rhs"
      pure s!"violation|{label}|pullback|commutes|pullback={pullback},row={row},lhs={lhs},rhs={rhs}"
  else
    throw "violation witness has no recognized variant"

private def normUnderconstraintWitness (label : String) (witness : Json) : Except String String := do
  let question ← fieldStr witness "question"
  let caseName ← fieldStr witness "case"
  let detail ← witness.getObjVal? "detail"
  match caseName with
  | "parallel_paths" => do
    let lhs ← strListField detail "lhs"
    let rhs ← strListField detail "rhs"
    pure s!"underconstraint|{label}|underconstraint|case=parallel_paths,lhs={renderStrings lhs},rhs={renderStrings rhs},question={question}"
  | "uncovered_inputs" => do
    let cls ← fieldStr detail "class"
    let examples ← numListField detail "examples"
    pure s!"underconstraint|{label}|underconstraint|case=uncovered_inputs,class={cls},examples={renderNats examples},question={question}"
  | other => throw s!"unknown domain question `{other}`"

/-- One normalized finding from the Rust envelope's `data.findings`. -/
private def normRustFinding (finding : Json) : Except String String := do
  let label ← fieldStr finding "label"
  let kind ← fieldStr finding "kind"
  let witness ← finding.getObjVal? "witness"
  match kind with
  | "violation" => normViolationWitness label witness
  | "dangling" => do
    let arrow ← fieldStr witness "arrow"
    let src ← numField witness "src"
    pure s!"dangling|{label}|dangling|arrow={arrow},src={src}"
  | "redundant" => do
    let derivable ← strListField witness "derivable"
    let reducing ← strListField witness "reducing"
    pure s!"redundant|{label}|redundant|derivable={renderStrings derivable},reducing={renderStrings reducing}"
  | "contradiction" => do
    let scope ← numListField witness "scope"
    pure s!"contradiction|{label}|contradiction|scope={renderNats scope}"
  | "underconstraint" => normUnderconstraintWitness label witness
  | other => throw s!"unknown finding kind `{other}`"

private def parseEnvelope (stdout : String) : Except String (List String) := do
  let json ← Json.parse stdout
  let ok ← json.getObjVal? "ok" >>= Json.getBool?
  if !ok then
    throw "envelope reports ok=false on a finding exit code"
  let data ← json.getObjVal? "data"
  let findings ← data.getObjVal? "findings" >>= Json.getArr?
  findings.toList.mapM normRustFinding

/-! ## Vendored expected sets -/

private structure Expected where
  route : String
  tables : List (Ident × List (Option RowId))
  dangling : List Instance.DanglingCell
  typingViolations : List Instance.TypingViolation
  outcomes : List String

private def decodeExpectedCells (json : Json) : Except String (List (Option RowId)) := do
  let cells ← json.getArr?
  cells.toList.mapM fun cell =>
    match cell with
    | .null => pure none
    | value => do
      let row ← value.getNat?
      pure (some row)

/-- One vendored outcome: a dangling finding, or a violation-shaped
outcome (P1's `hypothesis` kind carries the `hypothesized` label). -/
private def normExpectedOutcome (outcome : Json) : Except String String := do
  let kind ← fieldStr outcome "kind"
  match kind with
  | "dangling" => do
    let arrow ← fieldStr outcome "arrow"
    let src ← numField outcome "src"
    pure s!"dangling|executed|dangling|arrow={arrow},src={src}"
  | "violation" => normViolationWitness "executed" outcome
  | "hypothesis" => normViolationWitness "hypothesized" outcome
  | "redundant" => do
    let derivable ← strListField outcome "derivable"
    let reducing ← strListField outcome "reducing"
    pure s!"redundant|executed|redundant|derivable={renderStrings derivable},reducing={renderStrings reducing}"
  | "contradiction" => do
    let scope ← numListField outcome "scope"
    pure s!"contradiction|executed|contradiction|scope={renderNats scope}"
  | "underconstraint" => normUnderconstraintWitness "executed" outcome
  | other => throw s!"unknown expected outcome kind `{other}`"

private def parseExpected (json : Json) : Except String Expected := do
  let route ← fieldStr json "route"
  let tableEntries ←
    match json.getObjValD "tables" with
    | .null => pure []
    | value => do
      let fields ← value.getObj?
      pure fields.toList
  let tables ← tableEntries.mapM fun entry => do
    let cells ← decodeExpectedCells entry.2
    pure (entry.1, cells)
  let danglingNodes ←
    match json.getObjValD "dangling" with
    | .null => pure #[]
    | value => value.getArr?
  let dangling ← danglingNodes.toList.mapM fun node => do
    let arrow ← fieldStr node "arrow"
    let src ← numField node "src"
    pure ({ arrow := arrow, src := src } : Instance.DanglingCell)
  let violationNodes ←
    match json.getObjValD "typing_violations" with
    | .null => pure #[]
    | value => value.getArr?
  let typingViolations ← violationNodes.toList.mapM fun node => do
    let arrow ← fieldStr node "arrow"
    let src ← numField node "src"
    let value ← numField node "value"
    pure ({ arrow := arrow, src := src, value := value } : Instance.TypingViolation)
  let outcomeNodes ←
    match json.getObjValD "outcomes" with
    | .null => pure #[]
    | value => value.getArr?
  let outcomes ← outcomeNodes.toList.mapM normExpectedOutcome
  pure { route := route, tables := tables, dangling := dangling, typingViolations := typingViolations, outcomes := outcomes }

private def loadExpected (path : System.FilePath) : IO Expected := do
  if !(← path.pathExists) then
    throw (IO.userError s!"missing expected.json at {path}")
  let text ← IO.FS.readFile path
  match Json.parse text with
  | .error message => throw (IO.userError s!"{path}: invalid JSON: {message}")
  | .ok json =>
    match parseExpected json with
    | .ok expected => pure expected
    | .error message => throw (IO.userError s!"{path}: {message}")

/-! ## Workspace, corpus, and process plumbing -/

private partial def walkUp (dir : System.FilePath) (fuel : Nat) : IO (Option System.FilePath) := do
  if fuel == 0 then
    pure none
  else
    let hasCargo ← (dir / "Cargo.toml").pathExists
    let hasCli ← (dir / "crates" / "catherine-cli").isDir
    if hasCargo && hasCli then
      pure (some dir)
    else
      match dir.parent with
      | some parent => walkUp parent (fuel - 1)
      | none => pure none

private def findWorkspaceRoot : IO System.FilePath := do
  match ← IO.getEnv "LEAN_PARITY_ROOT" with
  | some value =>
    let path : System.FilePath := ⟨value⟩
    if ← path.pathExists then
      pure path
    else
      throw (IO.userError s!"LEAN_PARITY_ROOT `{value}` does not exist")
  | none =>
    match ← walkUp (← IO.currentDir) 40 with
    | some root => pure root
    | none =>
      throw (IO.userError "cannot locate the workspace root (Cargo.toml + crates/catherine-cli); set LEAN_PARITY_ROOT")

private partial def collectRelative (dir : System.FilePath) (pathPrefix : String) : IO (List String) := do
  let entries ← dir.readDir
  let mut found : List String := []
  for entry in entries do
    let relative := if pathPrefix.isEmpty then entry.fileName else pathPrefix ++ "/" ++ entry.fileName
    if ← entry.path.isDir then
      found := (← collectRelative entry.path relative) ++ found
    else if entry.fileName.endsWith ".ct.json" then
      found := relative :: found
  pure found

/-- The check-applicable corpus: `{katas,parity,probes}/**/*.ct.json`,
recursively enumerated and sorted. A corpus directory without those
sections is walked wholesale. -/
private def collectCorpus (corpus : System.FilePath) : IO (List String) := do
  let mut names : List String := []
  for directoryName in ["katas", "parity", "probes"] do
    let dir := corpus / directoryName
    if ← dir.isDir then
      names := (← collectRelative dir directoryName) ++ names
  if names.isEmpty then
    names ← collectRelative corpus ""
  pure (names.mergeSort lessString)

/-- The fixture's display name: parity documents collapse to
`parity/<case>`, everything else keeps its corpus-relative path. -/
def displayName (relative : String) : String :=
  match relative.splitOn "/" with
  | ["parity", name, "model.ct.json"] => s!"parity/{name}"
  | _ => relative

private def siblingExpected (relative : String) : Option String :=
  match relative.splitOn "/" with
  | ["parity", name, "model.ct.json"] => some s!"parity/{name}/expected.json"
  | _ => none

private def firstLine (text : String) : String :=
  match text.splitOn "\n" with
  | [] => ""
  | line :: _ => line

private def runCli (bin doc : String) : IO (UInt32 × String × String) := do
  let output ←
    IO.Process.output { cmd := bin, args := #["check", doc, "--json"] }
  pure (output.exitCode, output.stdout, output.stderr)

private def writeMirror (dir display : String) (json : Json) : IO Unit := do
  let relative := if display.endsWith ".json" then display else display ++ ".json"
  let file := System.FilePath.mk (dir ++ "/" ++ relative)
  match file.parent with
  | some parent => IO.FS.createDirAll parent
  | none => pure ()
  IO.FS.writeFile file json.pretty

private def mkDivergence (display : String) (rust lean : List String)
    (dispositions : List (String × Disposition)) : Divergence :=
  match dispositions.find? (fun entry => entry.1 == display) with
  | some entry =>
    { fixture := display, rust := rust, lean := lean, side := entry.2.side, beads := entry.2.beads }
  | none =>
    { fixture := display, rust := rust, lean := lean, side := "unattributed", beads := none }

private def compareBuild (display : String) (built : Instance.TypedInstance)
    (expected : Expected) : List BuildMismatch :=
  let expectedTables := renderTables expected.tables
  let leanTables := renderTables built.tables
  let tableMismatch :=
    if leanTables != expectedTables then
      [{ fixture := display, aspect := "tables", expected := expectedTables, lean := leanTables }]
    else []
  let expectedDangling := expected.dangling.map renderDanglingCell
  let leanDangling := built.dangling.map renderDanglingCell
  let danglingMismatch :=
    if leanDangling != expectedDangling then
      [{ fixture := display, aspect := "dangling", expected := expectedDangling, lean := leanDangling }]
    else []
  let expectedTyping := expected.typingViolations.map renderTypingViolation
  let leanTyping := built.typingViolations.map renderTypingViolation
  let typingMismatch :=
    if leanTyping != expectedTyping then
      [{ fixture := display, aspect := "typing_violations", expected := expectedTyping, lean := leanTyping }]
    else []
  tableMismatch ++ danglingMismatch ++ typingMismatch

/-- Project a document, round-trip it through the mirror JSON (so the
decoder the oracle consumes is exercised), and optionally emit the
mirror file. -/
private def projectDocument (options : Options) (corpus : System.FilePath)
    (display relative : String) : IO Mirror.MirrorDoc := do
  let text ← IO.FS.readFile (corpus / relative)
  let json ←
    match Json.parse text with
    | .ok json => pure json
    | .error message => throw (IO.userError s!"{display}: document is not JSON: {message}")
  let projected ←
    match Mirror.projectDocument display json with
    | .ok projected => pure projected
    | .error message => throw (IO.userError s!"harness projection failed: {message}")
  let mirrorJson := Mirror.toJson projected
  let reparsed ←
    match Json.parse mirrorJson.compress with
    | .ok reparsed => pure reparsed
    | .error message => throw (IO.userError s!"{display}: mirror JSON does not parse: {message}")
  let decoded ←
    match Mirror.decodeMirror reparsed with
    | .ok decoded => pure decoded
    | .error message => throw (IO.userError s!"{display}: mirror JSON does not decode: {message}")
  match options.emitMirror with
  | some dir => writeMirror dir display mirrorJson
  | none => pure ()
  pure decoded

private def leanFindings (projected : Mirror.MirrorDoc) : List String :=
  let built := Instance.build projected.schema projected.raw
  sortedFindings ((Check.check projected.schema built).map renderFinding)

/-- Process one corpus document. Harness-level problems throw; semantic
mismatches land in the returned outcome. -/
private def processDoc (options : Options) (corpus : System.FilePath) (bin : String)
    (relative : String) (dispositions : List (String × Disposition)) : IO DocOutcome := do
  let display := displayName relative
  let expected? : Option Expected ←
    match siblingExpected relative with
    | some relativeExpected => pure (some (← loadExpected (corpus / relativeExpected)))
    | none => pure none
  let (code, stdout, stderr) ← runCli bin (corpus / relative).toString
  let code := code.toNat
  if options.verbose then
    IO.eprintln s!"[lean-parity] {display}: catherine check exit {code}"
  if code == 1 then
    let line := firstLine stderr
    match expected? with
    | some expected =>
      if expected.route != "presentation" then
        throw (IO.userError s!"{display}: expected route `{expected.route}` but the Rust parser rejected the document: {line}")
      let projected ← projectDocument options corpus display relative
      let built := Instance.build projected.schema projected.raw
      let lean := leanFindings projected
      let rust := sortedFindings expected.outcomes
      let mismatches := if options.checkBuild then compareBuild display built expected else []
      if lean == rust then
        pure
          { display := display
            presentation := true
            skipped := some (display, line)
            triviallyAgreeing := if rust.isEmpty then some display else none
            findingsBearingAgreeing := !rust.isEmpty
            buildMismatches := mismatches }
      else
        pure
          { display := display
            presentation := true
            skipped := some (display, line)
            divergence := some (mkDivergence display rust lean dispositions)
            buildMismatches := mismatches }
    | none =>
      pure { display := display, skipped := some (display, line) }
  else if code == 0 || (code >= 3 && code <= 7) then
    let rust ←
      match parseEnvelope stdout with
      | .ok findings => pure (sortedFindings findings)
      | .error message => throw (IO.userError s!"{display}: malformed `check --json` envelope: {message}")
    let projected ← projectDocument options corpus display relative
    let built := Instance.build projected.schema projected.raw
    let lean := sortedFindings ((Check.check projected.schema built).map renderFinding)
    match expected? with
    | some expected =>
      if expected.route == "presentation" then
        throw (IO.userError s!"{display}: expected the Rust parser to reject this presentation fixture, but the CLI exited {code}")
      let mismatches := if options.checkBuild then compareBuild display built expected else []
      if lean == rust then
        pure
          { display := display
            checkable := true
            triviallyAgreeing := if rust.isEmpty then some display else none
            findingsBearingAgreeing := !rust.isEmpty
            buildMismatches := mismatches }
      else
        pure
          { display := display
            checkable := true
            divergence := some (mkDivergence display rust lean dispositions)
            buildMismatches := mismatches }
    | none =>
      if lean == rust then
        pure
          { display := display
            checkable := true
            triviallyAgreeing := if rust.isEmpty then some display else none
            findingsBearingAgreeing := !rust.isEmpty }
      else
        pure
          { display := display
            checkable := true
            divergence := some (mkDivergence display rust lean dispositions) }
  else
    throw (IO.userError s!"{display}: catherine check exited {code}: {firstLine stderr}")

/-! ## Dispositions -/

private def loadDispositions (root : System.FilePath) : IO (List (String × Disposition)) := do
  let path := root / "lean" / "parity-dispositions.json"
  if !(← path.pathExists) then
    pure []
  else
    let text ← IO.FS.readFile path
    let json ←
      match Json.parse text with
      | .ok json => pure json
      | .error message => throw (IO.userError s!"lean/parity-dispositions.json: invalid JSON: {message}")
    let entries ←
      match json.getObj? with
      | .ok entries => pure entries.toList
      | .error message => throw (IO.userError s!"lean/parity-dispositions.json: {message}")
    entries.mapM fun entry => do
      let side ←
        match entry.2.getObjVal? "side" >>= Json.getStr? with
        | .ok side => pure side
        | .error message => throw (IO.userError s!"lean/parity-dispositions.json[{entry.1}]: {message}")
      let beads :=
        match entry.2.getObjValD "beads" with
        | .str beads => some beads
        | _ => none
      pure (entry.1, ({ side := side, beads := beads } : Disposition))

/-! ## Report -/

private def mergeOutcome (summary : Summary) (outcome : DocOutcome) : Summary :=
  { total := summary.total + 1
    rustCheckable := summary.rustCheckable + (if outcome.checkable then 1 else 0)
    skipped :=
      match outcome.skipped with
      | some entry => summary.skipped ++ [entry]
      | none => summary.skipped
    presentation :=
      if outcome.presentation then summary.presentation ++ [outcome.display]
      else summary.presentation
    triviallyAgreeing :=
      match outcome.triviallyAgreeing with
      | some fixture => summary.triviallyAgreeing ++ [fixture]
      | none => summary.triviallyAgreeing
    findingsBearingAgreeing :=
      summary.findingsBearingAgreeing + (if outcome.findingsBearingAgreeing then 1 else 0)
    divergences :=
      match outcome.divergence with
      | some divergence => summary.divergences ++ [divergence]
      | none => summary.divergences
    buildMismatches := summary.buildMismatches ++ outcome.buildMismatches }

private def optionJson : Option String → Json
  | none => .null
  | some value => .str value

private def divergenceJson (divergence : Divergence) : Json :=
  Json.mkObj
    [ ("fixture", .str divergence.fixture)
    , ("rust", .arr (divergence.rust.map Json.str).toArray)
    , ("lean", .arr (divergence.lean.map Json.str).toArray)
    , ("side", .str divergence.side)
    , ("beads", optionJson divergence.beads) ]

private def mismatchJson (mismatch : BuildMismatch) : Json :=
  Json.mkObj
    [ ("fixture", .str mismatch.fixture)
    , ("aspect", .str mismatch.aspect)
    , ("expected", .arr (mismatch.expected.map Json.str).toArray)
    , ("lean", .arr (mismatch.lean.map Json.str).toArray) ]

private def reportJson (summary : Summary) (checkBuild : Bool) : Json :=
  Json.mkObj
    [ ("ok", .bool (summary.divergences.isEmpty && summary.buildMismatches.isEmpty))
    , ("check_build", .bool checkBuild)
    , ("corpus",
        Json.mkObj
          [ ("total", Json.num summary.total)
          , ("rust_checkable", Json.num summary.rustCheckable)
          , ("skipped_parse",
              .arr <| (summary.skipped.map fun entry =>
                Json.mkObj [("fixture", .str entry.1), ("stderr", .str entry.2)]).toArray)
          , ("lean_only_presentation", .arr (summary.presentation.map Json.str).toArray) ])
    , ("agreement",
        Json.mkObj
          [ ("trivially_agreeing", .arr (summary.triviallyAgreeing.map Json.str).toArray)
          , ("findings_bearing_agreeing", Json.num summary.findingsBearingAgreeing) ])
    , ("divergences", .arr (summary.divergences.map divergenceJson).toArray)
    , ("build_mismatches", .arr (summary.buildMismatches.map mismatchJson).toArray) ]

/-- A concise human rendering of the CLI's first stderr line: the error
code and message when the line is the JSON error envelope, the line
itself otherwise. The report keeps the verbatim first stderr line. -/
private def describeStderr (line : String) : String :=
  match Json.parse line with
  | .ok json =>
    match json.getObjVal? "data" with
    | .ok data =>
      match data.getObjVal? "message" >>= Json.getStr? with
      | .ok message =>
        match data.getObjVal? "code" >>= Json.getStr? with
        | .ok code => s!"{code}: {message}"
        | .error _ => message
      | .error _ => line
    | .error _ => line
  | .error _ => line

private def printHuman (summary : Summary) (verbose : Bool) : IO Unit := do
  for (fixture, line) in summary.skipped do
    IO.println s!"skipped_parse: {fixture} :: {describeStderr line}"
  for divergence in summary.divergences do
    IO.println s!"divergence: {divergence.fixture} rust={divergence.rust.length} lean={divergence.lean.length} side={divergence.side}"
    if verbose then
      for finding in divergence.rust do
        IO.eprintln s!"  rust {finding}"
      for finding in divergence.lean do
        IO.eprintln s!"  lean {finding}"
  for mismatch in summary.buildMismatches do
    IO.println s!"build_mismatch: {mismatch.fixture} {mismatch.aspect}"
    if verbose then
      for entry in mismatch.expected do
        IO.eprintln s!"  expected {entry}"
      for entry in mismatch.lean do
        IO.eprintln s!"  lean {entry}"
  IO.println
    s!"lean-parity: total={summary.total} rust_checkable={summary.rustCheckable} skipped_parse={summary.skipped.length} lean_only_presentation={summary.presentation.length} trivially_agreeing={summary.triviallyAgreeing.length} findings_bearing_agreeing={summary.findingsBearingAgreeing} divergences={summary.divergences.length} build_mismatches={summary.buildMismatches.length}"

/-! ## Entry point -/

private def usage : String :=
  "usage: lean-parity [--emit-mirror DIR] [--check-build] [--corpus DIR] [--verbose] [--report PATH]"

def run (args : List String) : IO UInt32 := do
  try
    let options ←
      match parseArgs args with
      | .ok options => pure options
      | .error message => throw (IO.userError s!"{message}\n{usage}")
    let root ← findWorkspaceRoot
    let bin :=
      match ← IO.getEnv "CATHERINE_BIN" with
      | some bin => bin
      | none => (root / "target" / "debug" / "catherine").toString
    if !(← (System.FilePath.mk bin).pathExists) then
      throw
        (IO.userError
          s!"catherine binary not found at `{bin}`; run `cargo build -p catherine-cli` or set CATHERINE_BIN")
    let corpus : System.FilePath :=
      match options.corpus with
      | some dir => ⟨dir⟩
      | none => root / "fixtures"
    if !(← corpus.isDir) then
      throw (IO.userError s!"corpus directory not found: {corpus}")
    let documents ← collectCorpus corpus
    if documents.isEmpty then
      throw (IO.userError s!"no `*.ct.json` documents found under {corpus}")
    let dispositions ← loadDispositions root
    let mut summary : Summary := {}
    for relative in documents do
      summary := mergeOutcome summary (← processDoc options corpus bin relative dispositions)
    printHuman summary options.verbose
    match options.report with
    | some path => IO.FS.writeFile ⟨path⟩ (reportJson summary options.checkBuild).pretty
    | none => pure ()
    let ok := summary.divergences.isEmpty && summary.buildMismatches.isEmpty
    pure (if ok then 0 else 1)
  catch error =>
    IO.eprintln s!"lean-parity: {error}"
    pure 2

def main (args : List String) : IO UInt32 := do
  if args.contains "--help" || args.contains "-h" then
    IO.println usage
    pure 0
  else
    run args

end Parity
end Catherine