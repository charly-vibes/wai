# Tasks: fix-stale-pipeline-run-tagging

## 1. Regression tests (red-first)

- [x] 1.1 `stale_pointer_to_completed_run_not_stamped`: pointer + completed run state → no `pipeline-run:`/`pipeline-step:` tags
- [x] 1.2 `stale_pointer_cleared_when_run_complete`: pointer + completed run state → pointer removed
- [x] 1.3 `pointer_to_missing_run_state_not_stamped_and_pointer_kept`: pointer + missing state → no tags, pointer kept
- [x] 1.4 Confirm control `step_tag_injected_when_pipeline_run_active` (incomplete run, pointer) stays green

## 2. Implementation

- [x] 2.1 Gate the pointer path of `build_tags` on run-state completeness (`active_pointer_run`, fail-closed)
- [x] 2.2 Self-heal: remove the pointer only when the run is proven complete

## 3. Validation

- [x] 3.1 Full `cargo test` green
- [x] 3.2 `openspec validate fix-stale-pipeline-run-tagging --strict`
- [x] 3.3 `just ci` green
