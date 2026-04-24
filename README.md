# Mira: Fallen Gates

### Testing
This is the normal cargo test command. Only builds with no test fails are allowed to merge.
> For faster cargo testing use the following command:
> ```bash
>   cargo nextest run -p game-testing
> ``` 
> This is the new and better way to test. This test framework gives you more information about the test.
> To install `cargo install cargo-nextest`
<br>
```bash
  cargo test -p game-testing
```
___
### Pull Request Title Guidelines
- Prefix must be one of: feat, fix, chore, test, base, ref
- Must include issue number like (#123)
- OR use "Release:" / "Beta:" without issue
- Example: `feat(#1): add new feature`
- Example: `fix(#42): fix bug`
- Example: `Release: 1.0.0`
- Example: `Beta: 0.9.0-beta.1`

### Pull Request Title Validation
A GitHub action is in place to validate PR titles against the guidelines. 
The action will fail the PR if the title does not conform to the specified format.

___

### Versioning
We use Cargo/Rust SemVer as the baseline versioning format.

- `DEV` state: use plain Rust/Cargo SemVer: `MAJOR.MINOR.PATCH`
- `BETA` state: use SemVer pre-release plus build metadata timestamp: `MAJOR.MINOR.PATCH-beta.N+YYYYMMDD.HHMMSS`
- `RELEASE` state: use SemVer release plus build metadata timestamp: `MAJOR.MINOR.PATCH+YYYYMMDD.HHMMSS`

Timestamp rule for `BETA` and `RELEASE`:
- Include date and time.
- Do not include milliseconds.

Examples:
- `0.4.0`
- `0.5.0-beta.1+20260424.153012`
- `1.0.0+20260424.153012`
