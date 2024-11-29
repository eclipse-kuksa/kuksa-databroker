# Wildcard Matching rules

*Note! This document applies to `sdv.databroker.v1` and `kuksa.val.v1`!*
*It also currently applies to `root` in `ListMetadata` in `kuksa.val.v2` but that may change in the future.*


* An empty pattern "" will match any signal.
* A pattern without any asterisk - a path in other words - matches either a signal directly or any signal that is a direct or indirect child of the branch with that path.
* An asterisk "`*`" at the end of a pattern will match any signal that is a direct child of the branch(es) identified by the preceding pattern.
* A double asterisk "`**`" at the end of a pattern matches any signal that is a direct or indirect child of the branch(es) identified by the preceding pattern.
* An asterisk "`*`" in the middle (or beginning) of a pattern matches any signal that has a branch (of any name) at that position.
* A double asterisk "`**`" in the middle (or beginning) of a pattern matches any signal that has zero or more branches at that position.

### Examples

| Path                | Matches                              |
|---------------------|--------------------------------------|
| `""`                | Everything                           |
| `"Vehicle"`         | Everything starting with `Vehicle`   |
| `"Vehicle.Speed"`   | `Vehicle.Speed` |
| `"Vehicle.Cabin.Sunroof"` | `Vehicle.Cabin.Sunroof.Position`<br>`Vehicle.Cabin.Sunroof.Shade.Position`<br>`Vehicle.Cabin.Sunroof.Shade.Switch`<br>`Vehicle.Cabin.Sunroof.Switch` |
| `"Vehicle.Cabin.Sunroof.**"` | `Vehicle.Cabin.Sunroof.Position`<br>`Vehicle.Cabin.Sunroof.Shade.Position`<br>`Vehicle.Cabin.Sunroof.Shade.Switch`<br>`Vehicle.Cabin.Sunroof.Switch`  |
| `"Vehicle.Cabin.Sunroof.*"` | `Vehicle.Cabin.Sunroof.Position`<br>`Vehicle.Cabin.Sunroof.Switch`  |
| `"Vehicle.Cabin.Sunroof.*.Position"` | `Vehicle.Cabin.Sunroof.Shade.Position` |
| `"**.Sunroof.*.Position"` | `Vehicle.Cabin.Sunroof.Shade.Position` |
| `"*.*.*.*.Position"` | `Vehicle.Cabin.Sunroof.Shade.Position` |
| `"Vehicle.Cabin.Sunroof.**.Position"` | `Vehicle.Cabin.Sunroof.Position`<br>`Vehicle.Cabin.Sunroof.Shade.Position` |
| `"**.Sunroof"`      | Nothing |
| `"**.Sunroof.**"` | `Vehicle.Cabin.Sunroof.Position`<br>`Vehicle.Cabin.Sunroof.Shade.Position`<br>`Vehicle.Cabin.Sunroof.Shade.Switch`<br>`Vehicle.Cabin.Sunroof.Switch`  |
| `"*.Sunroof"`       | Nothing|
| `"Sunroof"`         | Nothing|


### Exceptions
* The combination of double asterisk wildcard with multiple consecutive single asterisk wildcards is not supported.


| Path                | Does not Match                       |
|---------------------|--------------------------------------|
| `**.*.*.*.Position` | `Vehicle/Cabin/Door/Row2/PassengerSide/Shade/Position` <br> `Vehicle/Cabin/Seat/Row2/PassengerSide/Position` <br>`Vehicle/Cabin/Seat/Row1/Middle/Position` |
