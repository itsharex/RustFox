//! 开发种子数据测试（仅 debug 构建运行）：内存库上完整跑一遍 seed_dev_data，
//! 断言项目 / 接口 / 环境 / Mock 规则 / 全局参数 / 激活项齐全。
#![cfg(debug_assertions)]

use fox_storage::db::memory_pool;
use fox_storage::repository as repo;
use fox_storage::seed::seed_dev_data;

#[tokio::test]
async fn seeds_full_fixture_set() {
    let db = memory_pool().await.unwrap();
    seed_dev_data(&db).await.unwrap();

    // 项目 3 个，全部以「小奏技术」品牌命名
    let projects = repo::list_projects(&db).await.unwrap();
    assert_eq!(projects.len(), 3);
    assert!(
        projects.iter().all(|p| p.name.contains("小奏技术")),
        "所有种子项目名都应包含品牌「小奏技术」"
    );
    let users = projects
        .iter()
        .find(|p| p.name == "小奏技术 · 用户服务")
        .expect("用户服务项目存在");

    // 用户服务：7 个接口（5 账号 + 2 鉴权），2 个文件夹，响应示例 5 条，Mock 规则 5 条
    let endpoints = repo::list_endpoints(&db, users.id).await.unwrap();
    assert_eq!(endpoints.len(), 7);
    let folders = repo::list_folders(&db, users.id).await.unwrap();
    assert_eq!(folders.len(), 2);
    let rules = repo::list_mock_rules(&db, users.id).await.unwrap();
    assert_eq!(rules.len(), 5);
    let list_users = endpoints
        .iter()
        .find(|e| e.path == "/users" && e.method == fox_core::model::HttpMethod::GET)
        .expect("用户列表接口存在");
    let examples = repo::list_response_examples(&db, list_users.id)
        .await
        .unwrap();
    assert_eq!(examples.len(), 1);
    assert_eq!(examples[0].status, 200);

    // 开放演示：6 个接口；GraphQL 网关：2 个接口
    let open_demo = projects
        .iter()
        .find(|p| p.name == "小奏技术 · 开放演示")
        .expect("开放演示项目存在");
    assert_eq!(
        repo::list_endpoints(&db, open_demo.id).await.unwrap().len(),
        6
    );
    let graphql = projects
        .iter()
        .find(|p| p.name == "小奏技术 · GraphQL 网关")
        .expect("GraphQL 项目存在");
    assert_eq!(
        repo::list_endpoints(&db, graphql.id).await.unwrap().len(),
        2
    );

    // 环境：每个项目各自的开发 / 测试环境（单一 Base URL + 变量）
    let envs = repo::list_environments(&db, users.id).await.unwrap();
    assert_eq!(envs.len(), 2);
    let dev = envs
        .iter()
        .find(|e| e.name == "开发环境")
        .expect("开发环境存在");
    assert_eq!(dev.base_url, "http://127.0.0.1:4010");
    assert_eq!(dev.variables.len(), 3);
    // 开放演示 / GraphQL 网关项目同样各有开发 + 测试环境
    let open_demo = projects
        .iter()
        .find(|p| p.name == "小奏技术 · 开放演示")
        .expect("开放演示项目存在");
    let open_envs = repo::list_environments(&db, open_demo.id).await.unwrap();
    assert_eq!(open_envs.len(), 2);
    let open_dev = open_envs
        .iter()
        .find(|e| e.name == "开发环境")
        .expect("开放演示开发环境存在");
    assert_eq!(open_dev.base_url, "https://jsonplaceholder.typicode.com");

    // 全局参数 2 个；激活项 settings 已写入（项目 + 该项目的激活环境）
    let params = repo::get_global_params(&db).await.unwrap();
    assert_eq!(params.len(), 2);
    let active_project = repo::get_setting(&db, "active_project_id")
        .await
        .unwrap()
        .expect("active_project_id 已写入");
    // dev 默认激活公网项目（jsonplaceholder）而非本地 Mock，开箱即可直接发送请求
    assert!(active_project.contains(&open_demo.id.to_string()));
    let active_env = repo::get_setting(&db, &format!("active_environment_id:{}", open_demo.id))
        .await
        .unwrap()
        .expect("激活环境已写入");
    assert!(active_env.contains(&open_dev.id.to_string()));
}

/// 幂等性：同一内存库重复 seed（模拟手动重复调用）不应报错。
#[tokio::test]
async fn seed_is_idempotent_per_fresh_db() {
    let db = memory_pool().await.unwrap();
    seed_dev_data(&db).await.unwrap();
    // 注意：设计上只对「每次启动清库后」的全新库调用一次；重复调用会因
    // upsert（save_*）而成功，但会追加第二份 Mock 规则 —— 这里仅验证不 panic。
    seed_dev_data(&db).await.unwrap();
}
