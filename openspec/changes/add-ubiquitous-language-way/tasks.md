## 1. Spec and structure
- [x] 1.1 Add the new `ubiquitous-language` capability spec defining the progressive-disclosure resource tree under `.wai/resources/ubiquitous-language/`
- [x] 1.2 Define the canonical file roles (root index, shared terms, bounded-context files) and the agent loading convention

## 2. wai way recommendation
- [x] 2.1 Add a new `wai way` best-practice check for ubiquitous language resources
- [x] 2.2 Cover pass/info states for missing tree, skeleton tree, and fully configured tree
- [x] 2.3 Ensure output recommends the canonical `.wai/resources/ubiquitous-language/` location and progressive-disclosure layout

## 3. Skill scaffolding
- [x] 3.1 Extend the built-in skill template library with `ubiquitous-language`
- [x] 3.2 Update CLI/spec help so `wai add skill <name> --template ubiquitous-language` is documented as a valid template
- [x] 3.3 Ensure the generated skill directs agents to update the resource tree incrementally instead of creating one giant glossary file

## 4. Managed block guidance
- [x] 4.1 Update the managed block spec so generated instructions mention `.wai/resources/ubiquitous-language/` when present
- [x] 4.2 Instruct agents to read the root index first, then only the relevant bounded-context files

## 5. Validation
- [x] 5.1 Run `openspec validate add-ubiquitous-language-way --strict`
