use crate::builder::StateMachineBuilder;

#[test]
fn build_all_states() -> color_eyre::Result<()> {
    let mut builder = StateMachineBuilder::new();
    builder.load_builtin_state_files()?;
    builder.active_all_state_files()?;
    let _sm = builder.build()?;
    Ok(())
}