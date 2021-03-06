use super::*;
use crate::format_time;
use crate::helpers::*;
use vrp_core::models::examples::create_example_problem;

fn create_test_problem(limits: Option<VehicleLimits>) -> Problem {
    Problem {
        fleet: Fleet {
            vehicles: vec![VehicleType {
                vehicle_ids: vec!["some_real_vehicle".to_string()],
                limits,
                ..create_default_vehicle_type()
            }],
            profiles: create_default_profiles(),
        },
        ..create_empty_problem()
    }
}

fn create_test_solution(statistic: Statistic, stops: Vec<Stop>) -> Solution {
    Solution {
        tours: vec![Tour {
            vehicle_id: "some_real_vehicle".to_string(),
            type_id: "my_vehicle".to_string(),
            shift_index: 0,
            stops,
            statistic,
            ..create_empty_tour()
        }],
        ..create_empty_solution()
    }
}

parameterized_test! {can_check_shift_and_distance_limit, (max_distance, shift_time, actual, expected_result), {
    let expected_result = if let Err(prefix_msg) = expected_result {
        Err(format!(
            "{} violation, expected: not more than {}, got: {}, vehicle id 'some_real_vehicle', shift index: 0",
            prefix_msg, max_distance.unwrap_or_else(|| shift_time.unwrap()), actual,
        ))
    } else {
        Ok(())
    };
    can_check_shift_and_distance_limit_impl(max_distance, shift_time, actual, expected_result);
}}

can_check_shift_and_distance_limit! {
    case_01: (Some(10.), None, 11, Result::<(), _>::Err("max distance limit")),
    case_02: (Some(10.), None, 10, Result::<_, &str>::Ok(())),
    case_03: (Some(10.), None, 9, Result::<_, &str>::Ok(())),

    case_04: (None, Some(10.), 11, Result::<(), _>::Err("shift time limit")),
    case_05: (None, Some(10.), 10, Result::<_, &str>::Ok(())),
    case_06: (None, Some(10.), 9, Result::<_, &str>::Ok(())),

    case_07: (None, None, i64::max_value(), Result::<_, &str>::Ok(())),
}

pub fn can_check_shift_and_distance_limit_impl(
    max_distance: Option<f64>,
    shift_time: Option<f64>,
    actual: i64,
    expected: Result<(), String>,
) {
    let problem =
        create_test_problem(Some(VehicleLimits { max_distance, shift_time, tour_size: None, allowed_areas: None }));
    let solution =
        create_test_solution(Statistic { distance: actual, duration: actual, ..Statistic::default() }, vec![]);

    let result = check_limits(&CheckerContext::new(create_example_problem(), problem, None, solution));

    assert_eq!(result, expected);
}

#[test]
pub fn can_check_tour_size_limit() {
    let problem = create_test_problem(Some(VehicleLimits {
        max_distance: None,
        shift_time: None,
        tour_size: Some(2),
        allowed_areas: None,
    }));
    let solution = create_test_solution(
        Statistic::default(),
        vec![
            create_stop_with_activity(
                "departure",
                "departure",
                (0., 0.),
                3,
                (format_time(0.).as_str(), format_time(0.).as_str()),
                0,
            ),
            create_stop_with_activity(
                "job1",
                "delivery",
                (1., 0.),
                2,
                (format_time(1.).as_str(), format_time(1.).as_str()),
                1,
            ),
            create_stop_with_activity(
                "job2",
                "delivery",
                (2., 0.),
                1,
                (format_time(2.).as_str(), format_time(2.).as_str()),
                2,
            ),
            create_stop_with_activity(
                "job3",
                "delivery",
                (3., 0.),
                0,
                (format_time(3.).as_str(), format_time(3.).as_str()),
                3,
            ),
            create_stop_with_activity(
                "arrival",
                "arrival",
                (0., 0.),
                0,
                (format_time(6.).as_str(), format_time(6.).as_str()),
                6,
            ),
        ],
    );

    let result = check_limits(&CheckerContext::new(create_example_problem(), problem, None, solution));

    assert_eq!(
        result,
        Err("tour size limit violation, expected: not more than 2, got: 3, vehicle id 'some_real_vehicle', shift index: 0"
            .to_string())
    );
}
