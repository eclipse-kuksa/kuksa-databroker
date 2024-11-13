# KUKSA Use Cases


# Content
- [KUKSA Use Cases](#kuksa-use-cases)
- [Content](#content)
- [Use Cases status](#use-cases-status)
- [Use cases:](#use-cases)
  - [UC1](#uc1)
  - [UC2](#uc2)
  - [UC3](#uc3)
  - [UC4](#uc4)
  - [UC5](#uc5)
  - [UC6](#uc6)
  - [UC7](#uc7)
  - [UC8](#uc8)

# Use Cases status
| Description                               | Status                       |
|-------------------------------------------|------------------------------|
| Implemented and Verified                  | 🟢                           |
| Approved, Not Yet Implemented             | 🟡                           |
| Long Term Goal                            | 🔴                           |

# Use cases:

## UC1
    Title: Get sensor values.

    Status: 🟢

    Description: Signal consumer gets sensor and actuator values of signals.

**Primary Actor:** Signal consumer

**Secondary Actors:** Databroker, Provider, Vehicle Network

**Priority:** High

**Preconditions:**
 * Signal Consumer and Provider contain a valid authentication token to connect and perform calls to Databroker.
 * Provider can read from Vehicle Network.
 * Provider can publish signal values to Databroker.

**Special requirements:**

**Assumptions:**
* Singal Consumer should get exactly all signal values requested.

**Postconditions:**
* Signal Consumer receives a list with all signals requested in same order as requested.

**Sequence diagram:**
![Signal Consumer](../diagrams/consumer_get_values.svg)

**Basic Flow:**
1.  Provider opens bidirectional stream and starts publishing sensor and actuator values at the cycle time received from Vehicle network.
4.  Signal Consumer calls GetValues with signal paths/ids.
5.  Databroker sends the response with signal values back to the Signal Consumer.
9.	Use case finishes.

**Exceptions:**

## UC2
    Title: Signal consumber subscribes by id(int32) to all signal values.

    Status: 🟢

    Description: Signal Consumer subscribes to all sensor and actuator values by ids and gets notifications when values changed.

**Primary Actor:** Signal Consumer

**Secondary Actors:** Databroker, Provider, Vehicle Network

**Priority**: High

**Preconditions:**
 * Signal Consumer and Provider contain a valid authentication token to connect and perform calls to Databroker.
 * Provider can read from Vehicle Network.
 * Provider can constantly publish signal values to Databroker at a high frequency.

**Special requirements:**
 * The use case must meet a high frequency notification rate.

**Assumptions:**
 * Signal Consumer subscribes to all possible sensor and actuator values.
 * Provider porvides signals values to Databroker as fast as receives data from the Vehicle Network.

**Postconditions:**
 * A subscription is created, and the consumer receives updates for those signals where the state changes.

**Sequence diagram:**

![Signal Consumer Subscribes](../diagrams/consumer_subscribes.svg)

**Basic Flow:**
1.  Provider opens bidirectional stream and starts publishing sensor and actuator values at the cycle time received from Vehicle network.
2.  Signal Consumer calls list metadata to get all signals ids:
3.  Signal Consumer subscribes to all the sensor and actuator values by their ids.
4.  Databroker sends the current values stream back to the Signal Consumer.
5.  Databroker receives from Provider the new signal values and update its database.
6.  Databroker sends the changed values stream back to the Signal Consumer.
7.	Signal Consumer closes subscription to Databroker.
8.	Use case finishes.

**Exceptions:**


## UC3
    Title: The consumer wants to set the value of an actuator signal.

    Status: 🟢

    Description: Signal Consumer actuates on an actuator.

**Primary Actor:** Signal Consumer

**Secondary Actors:** Databroker, Provider, Vehicle Network

**Priority:** High

**Preconditions:**
 * Signal Consumer and Provider contain a valid authentication token to connect and perform calls to Databroker.
 * Provider can write to Vehicle Network.
 * No other Provider has claimed the ownership of the actuator to be actuated.

**Special requirements:**

**Assumptions:**
 * It does not necessarily mean that the actuator successfully updated its value to the desired new value. The entire chain is only responsible for forwarding the actuation request from Signal Consumer to the Vehicle Network.

**Postconditions:**
 * Provider forward an ack of receipt back to the Databroker immediately after the actuation request is forwarded.
 * Signal Consumer receives a response which indicates the operation was successfully forwarded.

**Sequence diagram:**
![Signal Consumer Actuate](../diagrams/consumer_actuate.svg)

**Basic Flow:**
1.  Provider opens bidirectional stream and sends a request to claim ownership of specific actuators.
2.  Databroker stores the claim request.
3.  Signal Consumer calls actuate with new actuator value.
4.  Databroker forwards the request to the corresponding provider.
5.  Provider receives the request and sends ack response back to Databroker.
6.  Databroker sends ack response back to the Signal Consumer.
7.  Provider sends the actuation request to the Vehicle Network.
8.	Use case finishes.

**Exceptions:**


## UC4
    Title: Signal Consumer actuates on multiple actuator.

    Status: 🟢

    Description: Signal Consumer actuates on multiple actuator.

**Primary Actor:** Signal Consumer

**Secondary Actors:** Databroker, Provider, Vehicle Network

**Priority:** High

**Preconditions:**
 * Signal Consumer and Providers contain a valid authentication token to connect and perform calls to Databroker.
 * Providers can write to Vehicle Network.
 * No other Provider has claimed the ownership of the actuator to be actuated.

**Special requirements:**

**Assumptions:**

**Postconditions:**
 * Providers forward an ack of receipt back to the Databroker immediately after the actuation request is forwarded.
 * Signal Consumer receives a response which indicates the operations were successfully forwarded.

**Sequence diagram:**
![Signal Consumer Actuate](../diagrams/consumer_actuate_multiple_providers.svg)

**Basic Flow:**
1.  Door Provider opens bidirectional stream and sends a ownership claim request of the Door actuator.
2.  Window Provider opens bidirectional stream and sends a ownership claim request of the Window actuator.
3.  Databroker stores the claims requests.
4.  Signal Consumer calls actuate with new Door and Window values.
5.  Databroker forwards the actuation request to the corresponding provider.
6.  Door Provider receives the Door actuation request and sends ack response back to Databroker.
7.  Window Provider receives the Window actuation request and sends ack response back to Databroker.
8.  Databroker sends ack response back to the Signal Consumer.
9.  Door Provider sends the Door actuation request to the Vehicle Network.
10. Window Provider sends the Window actuation request to the Vehicle Network.
11.	Use case finishes.

**Exceptions:**

## UC5
    Title: Signal Consumer and Provider get metadata of signals.

    Status: 🟢

    Description: Signal Consumer and Provider receives a list with metadata of VSS signals present in Databroker.

**Primary Actor:** Signal Consumer, Provider

**Secondary Actors:** Databroker

**Priority:** High

**Preconditions:**

**Special requirements:**

**Assumptions:**

**Postconditions:**

**Sequence diagram:**
![Signal Consumer Actuate](../diagrams/consumer_provider_list_metadata.svg)

**Basic Flow:**
1.  Signal Consumer calls list metadata to get all signals metadata.
2.  Provider calls list metadata to get all signals metadata.
3.	Use case finishes.

**Exceptions:**

## UC6
    Title: Signal Consumer and Provider get server info

    Status: 🟢

    Description: Signal Consumer and Provider get server info

**Primary Actor:** Signal Consumer, Provider

**Secondary Actors:** Databroker

**Priority:** High

**Preconditions:**

**Special requirements:**

**Assumptions:**

**Postconditions:**

**Sequence diagram:**
![Signal Consumer Actuate](../diagrams/consumer_provider_server_info.svg)

**Basic Flow:**
1.  Signal Consumer calls get server info.
2.  Provider calls get server info.
3.	Use case finishes.

**Exceptions:**


## UC7
    Title: Provider publishes signal values at high frequency.

    Status: 🟢

    Description: Provider publishes signals values to Databroker at a high frequency according to the cycle time from the Vehicle Network.

**Primary Actor:** Provider

**Secondary Actors:** Databroker, Vehicle Network

**Priority:** High

**Preconditions:**
* Provider can read from Vehicle Network.

**Special requirements:**
* Provider publishes signal values to Databroker atomically.

**Assumptions:**
* Provider has a list of valid signals with their ids(int32) that are present on Databroker.

**Postconditions:**
* Databroker stores on database all the sensor values.

**Sequence diagram:**
![Provider publish signals](../diagrams/provider_publish.svg)

**Basic Flow:**
1.  Provider start publishing at high frequency sensor and actuator values (by their ids(int32)) received from Vehicle network.
2.  Databroker send publish response back to provider.
3.	Use case finishes.

**Alternative Flows:**

**Exceptions:**

## UC8
    Title: Forward actuation request to Vehicle Netwrok

    Status: 🟢

    Description: Provider receives an actuator request to change an actuator value on the Vehicle Network.

**Primary Actor:** Provider

**Secondary Actors:** Databroker, Vehicle Network

**Priority:** High

**Preconditions:**

**Special requirements:**

**Assumptions:**
* Provider can stablish a connection with the Vehicle Network.
* There is an instance of Databroker up and running.
* Signal Consumer calls actuate with new actuator value.

**Postconditions:**

**Sequence diagram:**
![Provider received actuation](../diagrams/provider_recv_actuation.svg)


**Basic Flow:**
1.  Provider opens bidirectional stream and sends a claim actuators request.
2.  Databroker stores the claim request.
5.  Databroker forwards the actuation request to the corresponding provider.
6.  Provider sends the actuation request to the Vehicle Network.
7.  Provider sends ack response back to Databroker.
9.	Use case finishes.

**Exceptions:**
