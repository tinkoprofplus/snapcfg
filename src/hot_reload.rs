pub trait HotReloadable {
    fn apply_toml(&mut self, table: &toml::Table);

    fn apply_toml_str(&mut self, raw: &str) -> Result<(), toml::de::Error> {
        let table: toml::Table = toml::from_str(raw)?;
        self.apply_toml(&table);
        Ok(())
    }
}
