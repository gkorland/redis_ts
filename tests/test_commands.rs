extern crate redis;
extern crate redis_ts;

use redis::{Commands, Connection, Value};
use redis_ts::{
    TsAggregationType, TsCommands, TsFilterOptions, TsInfo, TsMget, TsMrange, TsOptions, TsRange,
};

use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn get_con() -> Connection {
    let client = redis::Client::open("redis://localhost/").unwrap();
    client.get_connection().expect("Failed to get connection!")
}

fn default_settings() -> TsOptions {
    TsOptions::default().retention_time(60000).label("a", "b")
}

fn sleep(ms: u64) {
    let millis = Duration::from_millis(ms);
    thread::sleep(millis);
}

#[test]
fn test_create_ts_no_options() {
    let _: () = get_con().del("test_ts_no_op").unwrap();
    let r: Value = get_con()
        .ts_create("test_ts_no_op", TsOptions::default())
        .unwrap();
    assert_eq!(Value::Okay, r);
    let info = get_con().ts_info("test_ts_no_op").unwrap();
    assert_eq!(info.retention_time, 0);
    assert_eq!(info.labels, vec![]);
}

#[test]
fn test_create_ts_retention() {
    let _: () = get_con().del("test_ts_ret").unwrap();
    let opts = TsOptions::default().retention_time(60000);
    let r: Value = get_con().ts_create("test_ts_ret", opts).unwrap();
    assert_eq!(Value::Okay, r);
    let info: TsInfo = get_con().ts_info("test_ts_ret").unwrap();
    assert_eq!(info.labels, vec![]);
    assert_eq!(info.retention_time, 60000);
}

#[test]
fn test_create_ts_labels() {
    let _: () = get_con().del("test_ts_lab").unwrap();
    let opts = TsOptions::default().label("a", "b");
    let r: Value = get_con().ts_create("test_ts_lab", opts).unwrap();
    assert_eq!(Value::Okay, r);
    let info: TsInfo = get_con().ts_info("test_ts_lab").unwrap();
    assert_eq!(info.labels, vec![("a".to_string(), "b".to_string())]);
    assert_eq!(info.retention_time, 0);
}

#[test]
fn test_create_ts_all() {
    let _: () = get_con().del("test_ts_all").unwrap();
    let opts = TsOptions::default()
        .retention_time(60000)
        .label("a", "b")
        .label("c", "d")
        .uncompressed(true);
    let r: Value = get_con().ts_create("test_ts_all", opts).unwrap();
    assert_eq!(Value::Okay, r);
}

#[test]
fn test_ts_add() {
    let _: () = get_con().del("test_ts_add").unwrap();
    let _: Value = get_con()
        .ts_create("test_ts_add", default_settings())
        .unwrap();
    let ts: u64 = get_con().ts_add("test_ts_add", 1234567890, 2.2).unwrap();
    assert_eq!(ts, 1234567890);
}

#[test]
fn test_ts_add_now() {
    let _: () = get_con().del("test_ts_add_now").unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let _: Value = get_con()
        .ts_create("test_ts_add_now", default_settings())
        .unwrap();
    let ts: u64 = get_con().ts_add_now("test_ts_add_now", 2.2).unwrap();
    assert!(now <= ts);
}

#[test]
fn test_ts_add_create() {
    let _: () = get_con().del("test_ts_add_create").unwrap();
    let ts: u64 = get_con()
        .ts_add_create("test_ts_add_create", 1234567890, 2.2, default_settings())
        .unwrap();
    assert_eq!(ts, 1234567890);
    let ts2: u64 = get_con()
        .ts_add_create("test_ts_add_create", "*", 2.3, default_settings())
        .unwrap();
    assert!(ts2 > ts);
}

#[test]
fn test_ts_madd() {
    let _: () = get_con().del("test_ts_madd").unwrap();
    let _: () = get_con().del("test_ts2_madd").unwrap();
    let _: Value = get_con()
        .ts_create("test_ts_madd", default_settings())
        .unwrap();
    let _: Value = get_con()
        .ts_create("test_ts2_madd", default_settings())
        .unwrap();

    let expected: Vec<u64> = vec![1234, 4321];
    let res: Vec<u64> = get_con()
        .ts_madd(&[("test_ts_madd", 1234, 1.0), ("test_ts2_madd", 4321, 2.0)])
        .unwrap();
    assert_eq!(expected, res);
}

#[test]
fn test_ts_incrby_now() {
    let _: () = get_con().del("test_ts_incrby_now").unwrap();
    let _: Value = get_con()
        .ts_create("test_ts_incrby_now", default_settings())
        .unwrap();

    let _: () = get_con().ts_incrby_now("test_ts_incrby_now", 1).unwrap();
    let v1: Option<(u64, f64)> = get_con().ts_get("test_ts_incrby_now").unwrap();
    assert_eq!(v1.unwrap().1, 1.0);
    sleep(1);
    let _: () = get_con().ts_incrby_now("test_ts_incrby_now", 5).unwrap();
    let v2: Option<(u64, f64)> = get_con().ts_get("test_ts_incrby_now").unwrap();
    assert_eq!(v2.unwrap().1, 6.0);
}

#[test]
fn test_ts_decrby_now() {
    let _: () = get_con().del("test_ts_decrby_now").unwrap();
    let _: Value = get_con()
        .ts_create("test_ts_decrby_now", default_settings())
        .unwrap();

    let _: () = get_con().ts_add_now("test_ts_decrby_now", 10).unwrap();
    let v1: Option<(u64, f64)> = get_con().ts_get("test_ts_decrby_now").unwrap();
    assert_eq!(v1.unwrap().1, 10.0);
    sleep(1);

    let _: () = get_con().ts_decrby_now("test_ts_decrby_now", 1).unwrap();
    let v1: Option<(u64, f64)> = get_con().ts_get("test_ts_decrby_now").unwrap();
    assert_eq!(v1.unwrap().1, 9.0);
    sleep(1);

    let _: () = get_con().ts_decrby_now("test_ts_decrby_now", 5).unwrap();
    let v2: Option<(u64, f64)> = get_con().ts_get("test_ts_decrby_now").unwrap();
    assert_eq!(v2.unwrap().1, 4.0);
}

#[test]
fn test_ts_incrby() {
    let _: () = get_con().del("test_ts_incrby").unwrap();
    let _: Value = get_con()
        .ts_create("test_ts_incrby", default_settings())
        .unwrap();

    let _: () = get_con().ts_incrby("test_ts_incrby", 123, 1).unwrap();
    let v1: Option<(u64, f64)> = get_con().ts_get("test_ts_incrby").unwrap();
    assert_eq!(v1.unwrap(), (123, 1.0));

    let _: () = get_con().ts_incrby("test_ts_incrby", 1234, 5).unwrap();
    let v2: Option<(u64, f64)> = get_con().ts_get("test_ts_incrby").unwrap();
    assert_eq!(v2.unwrap(), (1234, 6.0));
}

#[test]
fn test_ts_decrby() {
    let _: () = get_con().del("test_ts_decrby").unwrap();
    let _: Value = get_con()
        .ts_create("test_ts_decrby", default_settings())
        .unwrap();
    let _: () = get_con().ts_add("test_ts_decrby", 12, 10).unwrap();
    let v1: Option<(u64, f64)> = get_con().ts_get("test_ts_decrby").unwrap();
    assert_eq!(v1.unwrap(), (12, 10.0));

    let _: () = get_con().ts_decrby("test_ts_decrby", 123, 1).unwrap();
    let v1: Option<(u64, f64)> = get_con().ts_get("test_ts_decrby").unwrap();
    assert_eq!(v1.unwrap(), (123, 9.0));

    let _: () = get_con().ts_decrby("test_ts_decrby", 1234, 5).unwrap();
    let v2: Option<(u64, f64)> = get_con().ts_get("test_ts_decrby").unwrap();
    assert_eq!(v2.unwrap(), (1234, 4.0));
}

#[test]
fn test_ts_incrby_create() {
    let _: () = get_con().del("test_ts_incrby_create").unwrap();

    let _: () = get_con()
        .ts_incrby_create("test_ts_incrby_create", 123, 1, default_settings())
        .unwrap();
    let v1: Option<(u64, f64)> = get_con().ts_get("test_ts_incrby_create").unwrap();
    assert_eq!(v1.unwrap(), (123, 1.0));

    let _: () = get_con()
        .ts_incrby_create("test_ts_incrby_create", 1234, 5, default_settings())
        .unwrap();
    let v2: Option<(u64, f64)> = get_con().ts_get("test_ts_incrby_create").unwrap();
    assert_eq!(v2.unwrap(), (1234, 6.0));
}

#[test]
fn test_ts_decrby_create() {
    let _: () = get_con().del("test_ts_decrby_create").unwrap();

    let _: () = get_con()
        .ts_decrby_create("test_ts_decrby_create", 123, 1, default_settings())
        .unwrap();
    let v1: Option<(u64, f64)> = get_con().ts_get("test_ts_decrby_create").unwrap();
    assert_eq!(v1.unwrap(), (123, -1.0));

    let _: () = get_con()
        .ts_decrby_create("test_ts_decrby_create", 1234, 5, default_settings())
        .unwrap();
    let v2: Option<(u64, f64)> = get_con().ts_get("test_ts_decrby_create").unwrap();
    assert_eq!(v2.unwrap(), (1234, -6.0));
}

#[test]
fn test_ts_create_delete_rule() {
    let _: () = get_con().del("test_ts_create_delete_rule").unwrap();
    let _: () = get_con().del("test_ts_create_delete_rule2").unwrap();
    let _: Value = get_con()
        .ts_create("test_ts_create_delete_rule", default_settings())
        .unwrap();
    let _: Value = get_con()
        .ts_create("test_ts_create_delete_rule2", default_settings())
        .unwrap();
    let _: () = get_con()
        .ts_createrule(
            "test_ts_create_delete_rule",
            "test_ts_create_delete_rule2",
            TsAggregationType::Avg(5000),
        )
        .unwrap();

    let info: TsInfo = get_con().ts_info("test_ts_create_delete_rule").unwrap();
    assert_eq!(
        info.rules,
        vec![(
            "test_ts_create_delete_rule2".to_string(),
            5000,
            "AVG".to_string()
        )]
    );

    let _: () = get_con()
        .ts_deleterule("test_ts_create_delete_rule", "test_ts_create_delete_rule2")
        .unwrap();
    let info: TsInfo = get_con().ts_info("test_ts_create_delete_rule").unwrap();
    assert_eq!(info.rules, vec![]);
}

#[test]
fn test_ts_get() {
    let _: () = get_con().del("test_ts_get").unwrap();
    let _: Value = get_con()
        .ts_create("test_ts_get", default_settings())
        .unwrap();
    let _: () = get_con().ts_add("test_ts_get", 1234, 2.0).unwrap();
    let res: Option<(u64, f64)> = get_con().ts_get("test_ts_get").unwrap();
    assert_eq!(Some((1234, 2.0)), res);
}

#[test]
fn test_ts_mget() {
    let _: () = get_con().del("test_ts_mget").unwrap();
    let _: () = get_con().del("test_ts_mget2").unwrap();
    let _: () = get_con().del("test_ts_mget3").unwrap();
    let opts: TsOptions = TsOptions::default().label("l", "mget");
    let _: Value = get_con().ts_create("test_ts_mget", opts.clone()).unwrap();
    let _: Value = get_con().ts_create("test_ts_mget2", opts.clone()).unwrap();
    let _: Value = get_con().ts_create("test_ts_mget3", opts.clone()).unwrap();
    let _: () = get_con()
        .ts_madd(&[
            ("test_ts_mget", 12, 1.0),
            ("test_ts_mget", 123, 2.0),
            ("test_ts_mget", 1234, 3.0),
            ("test_ts_mget2", 21, 1.0),
            ("test_ts_mget2", 321, 2.0),
            ("test_ts_mget2", 4321, 3.0),
        ])
        .unwrap();
    let res: TsMget<u64, f64> = get_con()
        .ts_mget(
            TsFilterOptions::default()
                .equals("l", "mget")
                .with_labels(true),
        )
        .unwrap();

    assert_eq!(res.values.len(), 3);
    assert_eq!(res.values[0].value, Some((1234, 3.0)));
    assert_eq!(res.values[1].value, Some((4321, 3.0)));
    assert_eq!(res.values[2].value, None);
}

#[test]
fn test_ts_get_ts_info() {
    let _: () = get_con().del("test_ts_get_ts_info").unwrap();
    let _: Value = get_con()
        .ts_create("test_ts_get_ts_info", default_settings())
        .unwrap();
    let _: () = get_con()
        .ts_add("test_ts_get_ts_info", "1234", 2.0)
        .unwrap();
    let info: TsInfo = get_con().ts_info("test_ts_get_ts_info").unwrap();
    assert_eq!(info.total_samples, 1);
    assert_eq!(info.first_timestamp, 1234);
    assert_eq!(info.last_timestamp, 1234);
    assert_eq!(info.chunk_count, 1);
    assert_eq!(info.labels, vec![("a".to_string(), "b".to_string())]);
}

#[test]
fn test_ts_range() {
    let _: () = get_con().del("test_ts_range").unwrap();
    let _: () = get_con().del("test_ts_range2").unwrap();
    let _: () = get_con()
        .ts_create("test_ts_range", default_settings())
        .unwrap();
    let _: () = get_con()
        .ts_create("test_ts_range2", default_settings())
        .unwrap();
    let _: () = get_con()
        .ts_madd(&[
            ("test_ts_range", 12, 1.0),
            ("test_ts_range", 123, 2.0),
            ("test_ts_range", 1234, 3.0),
        ])
        .unwrap();

    let res: TsRange<u64, f64> = get_con()
        .ts_range("test_ts_range", "-", "+", None::<usize>, None)
        .unwrap();
    assert_eq!(res.values, vec![(12, 1.0), (123, 2.0), (1234, 3.0)]);

    let one_res: TsRange<u64, f64> = get_con()
        .ts_range("test_ts_range", "-", "+", Some(1), None)
        .unwrap();
    assert_eq!(one_res.values, vec![(12, 1.0)]);

    let range_res: TsRange<u64, f64> = get_con()
        .ts_range("test_ts_range", 12, 123, None::<usize>, None)
        .unwrap();
    assert_eq!(range_res.values, vec![(12, 1.0), (123, 2.0)]);

    let sum: TsRange<u64, f64> = get_con()
        .ts_range(
            "test_ts_range",
            12,
            123,
            None::<usize>,
            Some(TsAggregationType::Sum(10000)),
        )
        .unwrap();
    assert_eq!(sum.values, vec![(0, 3.0)]);

    let res: TsRange<u64, f64> = get_con()
        .ts_range("test_ts_range2", "-", "+", None::<usize>, None)
        .unwrap();
    assert_eq!(res.values, vec![]);
}

#[test]
fn test_ts_mrange() {
    let _: () = get_con().del("test_ts_mrange").unwrap();
    let _: () = get_con().del("test_ts_mrange2").unwrap();
    let opts: TsOptions = TsOptions::default().label("l", "mrange");
    let _: () = get_con().ts_create("test_ts_mrange", opts.clone()).unwrap();
    let _: () = get_con()
        .ts_create("test_ts_mrange2", opts.clone())
        .unwrap();
    let _: () = get_con()
        .ts_madd(&[
            ("test_ts_mrange", 12, 1.0),
            ("test_ts_mrange", 123, 2.0),
            ("test_ts_mrange", 1234, 3.0),
            ("test_ts_mrange2", 21, 1.0),
            ("test_ts_mrange2", 321, 2.0),
            ("test_ts_mrange2", 4321, 3.0),
        ])
        .unwrap();

    let res: TsMrange<u64, f64> = get_con()
        .ts_mrange(
            "-",
            "+",
            None::<usize>,
            None,
            TsFilterOptions::default()
                .equals("l", "mrange")
                .with_labels(true),
        )
        .unwrap();
    assert_eq!(res.values.len(), 2);
    assert_eq!(
        res.values[1].values,
        vec![(21, 1.0), (321, 2.0), (4321, 3.0)]
    );
    assert_eq!(res.values[0].key, "test_ts_mrange");
    assert_eq!(res.values[1].key, "test_ts_mrange2");
    assert_eq!(
        res.values[0].labels,
        vec![("l".to_string(), "mrange".to_string())]
    );

    let res2: TsMrange<u64, f64> = get_con()
        .ts_mrange(
            "-",
            "+",
            None::<usize>,
            None,
            TsFilterOptions::default()
                .equals("none", "existing")
                .with_labels(true),
        )
        .unwrap();
    assert!(res2.values.is_empty());
}

#[test]
fn test_ts_queryindex() {
    let _: () = get_con().del("test_ts_queryindex").unwrap();
    let _: Value = get_con()
        .ts_create("test_ts_queryindex", default_settings())
        .unwrap();
    let _: () = get_con().ts_add("test_ts_queryindex", "1234", 2.0).unwrap();
    let index: Vec<String> = get_con()
        .ts_queryindex(TsFilterOptions::default().equals("a", "b"))
        .unwrap();
    assert!(index.contains(&"test_ts_queryindex".to_string()));
}
