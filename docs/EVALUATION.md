# Evaluation and Continuous Improvement

Swarm includes an integrated LLM-as-a-Judge evaluation service backed by `redb`.

The goal is to make evaluation part of the runtime rather than an external afterthought.

## What is stored

Evaluation records can include:

- request ID;
- agent identity;
- input;
- execution output;
- judge assessment;
- quality score;
- critique;
- timestamp.

The current database is stored at:

```text
swarm/database/evaluation_db.redb
```

## Runtime flow

```text
Request
  ↓
Planning
  ↓
Agent Execution
  ↓
Tool Calls
  ↓
Result
  ↓
Judge
  ↓
Evaluation Record
```

## Why this matters

Gateway observability typically answers questions such as:

```text
Which model was called?
How many tokens were used?
How long did it take?
```

Agent evaluation should eventually answer:

```text
Which plan was selected?
Which agent executed each step?
Which tool was called?
Why did the workflow fail?
How was the output scored?
Did a retry improve the result?
```

## Current endpoints

Log an evaluation:

```text
POST /log
```

Inspect evaluation history:

```text
GET /evaluations
```

## Future direction

Evaluation data can support:

- regression testing;
- prompt comparison;
- model benchmarking;
- quality monitoring;
- routing optimization;
- supervised fine-tuning datasets;
- preference datasets;
- continuous improvement loops.

A key long-term goal is to combine evaluation with routing.

For example:

```text
Task Type
   ↓
Historical Evaluation Data
   ↓
Best Model / Provider
   ↓
Execution
   ↓
New Evaluation
```

This can allow Swarm to optimize not only for latency and cost, but also for observed task quality.
