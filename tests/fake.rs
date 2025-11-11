#[cfg(test)]
mod tests {
    use fake::faker::name::raw::*;
    use fake::locales::*;
    use fake::Fake;

    #[test]
    fn test_user_creation() {
        // 只在测试中使用 fake 生成测试数据
        let test_name: String = Name(ZH_CN).fake();
        let test_age: u32 = (18..60).fake();
        println!("user name: {}", test_name);
        println!("user age: {}", test_age);
    }

    #[test]
    fn test_multiple_users() {
        for _ in 0..5 {
            let name: String = Name(EN).fake();
            let age: u32 = (18..60).fake();
            println!("user name {}", name);
            assert!(age >= 18 && age < 60);
        }
    }
}
