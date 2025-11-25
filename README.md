superpoweredCV (Rust skeleton)
==============================

This crate starts implementing the spec for superpoweredCV and the PDF prompt-injection red-team module. It provides:
- Canonical profile data structures matching the LinkedIn ingest/ATS metadata model.
- Injection templates and profile configs for the PDF red-team engine.
- Scenario and pipeline definitions plus a minimal executor that wires profiles + templates into scenario reports.

Running the example
-------------------

The binary prints a JSON report stub for a sample scenario:

```bash
cargo run
```

Key modules
-----------
- `src/profile.rs` - `UserProfile` schema and related structs for contact, experience, education, skills, and ATS metadata.
- `src/templates.rs` - built-in injection templates drawn from the spec (soft bias, overrides, control blocks, etc.).
- `src/red_team.rs` - injection profile configs, scenario definition, a lightweight `RedTeamEngine`, and a `PipelineExecutor` trait for integrating ATS/AI evaluation.
- `src/pipeline.rs` - pipeline and metric definitions for AI/ATS scoring and logging capture.
- `src/pdf.rs` - PDF mutator abstraction plus a stub mutator that emits placeholder artifacts with hashes.

Next steps
----------
- Implement real PDF mutation for each profile (visible blocks, low-visibility text, off-page layers, underlay text, structural fields, padding, inline job ads) wired into `PdfMutator`.
- Integrate the ATS/AI simulation pipeline and metric computation so `ScenarioReport` contains real impacts via a concrete `PipelineExecutor`.
- Add LaTeX rendering hooks and persist variant hashes.
