use snowflake::SnowflakeIdGenerator;


pub fn generate_id() -> i64 {
    // TODO: find a better way of assigning machine and node IDs
    let mut generator = SnowflakeIdGenerator::new(
        1,
        1,
    );
    let id = generator.real_time_generate();
    return id;
}
