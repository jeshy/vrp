# Unassigned jobs

When job cannot be assigned, it goes to the list of unassigned jobs:

```json
{{#include ../../../../../examples/data/pragmatic/basics/unassigned.unreachable.solution.json:113:123}}
```

Each item in this list has job id, reason code and description.


## Reasons of unassigned jobs

|         code            |                        description                             |                  possible action                        |
|-------------------------|----------------------------------------------------------------|---------------------------------------------------------|
| NO_REASON_FOUND         | `unknown`                                                      |                                                         |
| SKILL_CONSTRAINT        | `cannot serve required skill`                                  | allocate more vehicles with given skill?                |
| TIME_WINDOW_CONSTRAINT  | `cannot be visited within time window`                         | allocate more vehicles, relax time windows, etc.?       |
| CAPACITY_CONSTRAINT     | `does not fit into any vehicle due to capacity`                | allocate more vehicles?                                 |
| REACHABLE_CONSTRAINT    | `location unreachable`                                         | change job location to routable place?                  |
| MAX_DISTANCE_CONSTRAINT | `cannot be assigned due to max distance constraint of vehicle` | allocate more vehicles?                                 |
| SHIFT_TIME_CONSTRAINT   | `cannot be assigned due to shift time constraint of vehicle`   | allocate more vehicles?                                 |
| BREAK_CONSTRAINT        | `break is not assignable`                                      | correct break location or/and time window?              |
| LOCKING_CONSTRAINT      | `cannot be served due to relation lock`                        | review relations?                                       |
| PRIORITY_CONSTRAINT     | `cannot be served due to priority`                             | allocate more vehicles, relax priorities?               |
| AREA_CONSTRAINT         | `cannot be assigned due to area constraint`                    | make sure that jobs inside allowed areas                |
| DISPATCH_CONSTRAINT     | `cannot be assigned due to vehicle dispatch`                   | make sure that vehicle dispatch definition is correct   |
| TOUR_SIZE_CONSTRAINT    | `cannot be assigned due to tour size constraint of vehicle`    | make sure that there are enough vehicles to serve jobs  |


## Example

An example of problem with unassigned jobs can be found [here](../../../examples/pragmatic/basics/unassigned.md).
