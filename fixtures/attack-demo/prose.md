# Prose requirements — library lending desk (the LLM's only input)

A lending desk tracks physical copies of books and the loans of those
copies to members. Every loan is for exactly one physical copy, and every
copy is a copy of exactly one catalog book. When a loan is recorded, the
desk writes down both which copy was handed out and (for shelf reports)
which catalog book the loan counts against — these must always agree:
the book a loan counts against is exactly the book of the copy that was
handed out. Each loan is taken out by exactly one member.
