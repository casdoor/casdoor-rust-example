mod util;

use casdoor_rust_sdk::{CasdoorConfig, CasdoorUser, UserService};

/// This is a sample code for casdoor-rust-sdk.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // must pass a absolute path to the config file.
    let conf = CasdoorConfig::from_toml(util::abs_path("conf.toml")?.as_str())?;
    let user_service = UserService::new(&conf);

    // get user count.
    // i have two users in my database.
    // may you just one.
    let count = user_service.get_user_count("0".to_string()).await?;
    assert_eq!(count, 2);

    // get user info.
    let user = user_service.get_user("admin".to_string()).await?;
    println!("{:#?}", user);
    assert_eq!(user.display_name.to_lowercase(), "admin");

    // create user.
    let new_user = CasdoorUser {
        name: "added_user".to_string(),
        ..user
    };

    let code = user_service.add_user(new_user).await?;
    assert_eq!(code, 200);
    // get user count. should be 3.
    let count = user_service.get_user_count("0".to_string()).await?;
    assert_eq!(count, 3);

    // delete user.
    let del_user = user_service.get_user("added_user".to_string()).await?;
    let code = user_service.delete_user(del_user).await?;
    assert_eq!(code, 200);
    // get user count. should be 2.
    let count = user_service.get_user_count("0".to_string()).await?;
    assert_eq!(count, 2);

    Ok(())
}
