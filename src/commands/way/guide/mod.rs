//! `wai way <topic>` guides, one file per topic.

mod ai;
mod ci;
mod code_quality;
mod coverage;
mod devxp;
mod docs;
mod gh;
mod hooks;
mod issues;
mod specs;

pub(crate) use ai::guide_ai;
pub(crate) use ci::guide_ci;
pub(crate) use code_quality::guide_code_quality;
pub(crate) use coverage::guide_coverage;
pub(crate) use devxp::guide_devxp;
pub(crate) use docs::guide_docs;
pub(crate) use gh::guide_gh;
pub(crate) use hooks::guide_hooks;
pub(crate) use issues::guide_issues;
pub(crate) use specs::guide_specs;
