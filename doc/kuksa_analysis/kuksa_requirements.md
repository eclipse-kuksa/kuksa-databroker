# Rerequirements

# Content
- [Rerequirements](#rerequirements)
- [Content](#content)
- [Requirements status](#requirements-status)
- [Functional requirements](#functional-requirements)
  - [As Signal Consumer](#as-signal-consumer)
    - [FR1-ASC](#fr1-asc)
    - [FR2-ASC](#fr2-asc)
    - [FR3-ASC](#fr3-asc)
    - [FR4-ASC](#fr4-asc)
    - [FR5-ASC](#fr5-asc)
    - [FR6-ASC](#fr6-asc)
    - [FR7-ASC](#fr7-asc)
    - [FR8-ASC](#fr8-asc)
  - [As Databroker](#as-databroker)
    - [FR1-AD](#fr1-ad)
    - [FR2-AD](#fr2-ad)
    - [FR3-AD](#fr3-ad)
    - [FR4-AD](#fr4-ad)
    - [FR5-AD](#fr5-ad)
  - [As Provider](#as-provider)
    - [FR1-AP](#fr1-ap)
    - [FR2-AP](#fr2-ap)
    - [FR3-AP](#fr3-ap)
    - [FR4-AP](#fr4-ap)
    - [FR5-AP](#fr5-ap)
- [Non-Functional Requirements](#non-functional-requirements)
- [Domain requirements](#domain-requirements)

# Requirements status
| Description                               | Status                       |
|-------------------------------------------|------------------------------|
| Implemented and Verified                  | 🟢                           |
| Approved, Not Yet Implemented             | 🟡                           |
| Long Term Goal                            | 🔴                           |

# Functional requirements

## As Signal Consumer
### FR1-ASC
    Title: Single access service Point for Signal Consumer

    Status: 🟢

    Description: The Signal Consumer shall have a single service point accessible.

### FR2-ASC
    Title: Uniform retrieval of vehicle and environmental Signal Values

    Status: 🟢

    Description: The Signal Consumer shall get sensor, actuator and attributes values of the vehicle signals and its environment(air temperature, etc) in a uniform manner.
      1.  IF there is an error
          * THEN the Signal Consumer shall not receive any signal value, just one single error with detailed error information
          * ELSE the Signal Consumer shall receive exactly all the signals’ values requested.

          n. A signal can have value out of the set of the defined value restriction/data type and its 'value' can be explicitly 'None', meaning the Signal exists but no value is present.

      2.  The Signal Consumer only shall get values for signal to which it has permission to.
      3.  The Signal Consumer shall provide the paths or ids(int32) of the requested signals.

### FR3-ASC
    Title: Subscription and high frequency notification for Signal Value changes

    Status: 🟢

    Description: The Signal Consumer shall be able to subscribe to sensor or actuator values of the vehicle signals and get immediate notifications when any of the signal values change.
      *  IF there is an error
          THEN the Signal Consumer shall not receive any signal value, just one single error with detailed error information
          ELSE the Signal Consumer shall receive exactly all the signals’ values requested that have change.
      *  The Signal Consumer only shall get values for signal to which it has permission to.
      *  The Signal Consumer shall provide the paths or the ids(int32) of the requested signals.

### FR4-ASC
    Title: Filtered interval subscription for Signal Values

    Status: 🔴

    Description: The Signal Consumer shall subscribe and be able to apply a filter to receive a signal values with an interval of x milliseconds.
      *  IF there is an error
          THEN the Signal Consumer shall not receive any signal value, just one single error with detailed error information
          ELSE the Signal Consumer shall receive exactly all the signals’ values requested.
      *  The Signal Consumer only shall get values for signal to which it has permission to.
      *  The Signal Consumer shall provide the paths or the ids(int32) of the requested signals.

### FR5-ASC
    Title: Accessing static metadata of signals

    Status: 🟢

    Description: A Signal Consumer shall be able to get static metadata from signals.
      *  IF there is an error
            THEN the Signal Consumer shall not receive any signal metadata, just one single error with detailed error information
            ELSE the Signal Consumer shall receive exactly all the signals’ metadata requested.

      * All sensors, actuators, and attributes values for which a Signal Consumer has permission to.
      * The Signal Consumer shall provide the path, paths or wildcard of the signals.

### FR6-ASC
    Title: Actuation of actuator value with Databroker forwarded acknowledgment.

    Status: 🟢

    Description: The Signal Consumer shall be able to actuate the value of an actuator. This value should be forwarded to the actuator's provider if the provider is available, provider to Vehicle Network and get an acknowledgment response back.
      * Databroker should not store the provided value but only forward it to the provider.

      * IF no provider is connected
          THEN Signal Consumer shall receive an error that no provider is available.
          ELSE IF databroker successfully forwarded the value to the provider
            THEN Signal Consumer shall receive an acknowledgement of receipt.
      * IF provided signal path is not an actuator Signal Consumer should receive an error.

### FR7-ASC
    Title: Actuate multiple actuators simultaneously with Databroker forwarded acknowledgment.

    Status: 🟢

    Description: The Signal Consumer shall be able to actuate the values of multiple actuators simultaneously. These values should be forwarded to the corresponding actuators' providers only if all providers are available.
      * Databroker should not store the provided value but only forward them to the providers.

      * IF any provider is not connected
          THEN Signal Consumer shall receive an error that no provider is available.
          ELSE IF databroker successfully forwarded the values to all providers
            THEN Signal Consumer shall receive an acknowledgement of receipt.
      * IF provided signal path is not an actuator Signal Consumer should receive an error.

### FR8-ASC
    Title: Provider availability detection for Signal Consumer

    Status: 🔴

    Description: The Signal Consumer shall be able to know if there is a provider up and running.

## As Databroker
### FR1-AD
    Title: Handling of COVESA Vehicle Signal Specification (VSS) syntax by Databroker

    Status: 🟢

    Description: The Databroker shall handle catalogs of signals described by the syntax as defined by the COVESA Vehicle Signal Specification (VSS). This relates to all aspects of the VSS syntax definition, which is also called VSS rule set. This implies that the Databroker can handle the signal catalog as defined by the COVESA VSS.

### FR2-AD
    Title: Support for VSS metadata elements by Databroker

    Status: 🟢

    Description: The Databroker shall support at least those metadata elements as defined by the VSS rule set.

### FR3-AD
    Title: Consumer subscription management by Databroker

    Status: 🟢

    Description: The Databroker shall keep a local record of all subscriptions of signal consumers.
      * The Databroker shall add or remove subscriptions to a subscription pool according to the subscription requests of the Signal Consumer.

### FR4-AD
    Title: Actuator ownership claim management by Databroker

    Status: 🟢

    Description: The Databroker shall maintain a local record of all actuator ownership claims made by signal providers.
      * The Databroker shall manage an "ownership claim actuation pool," adding or removing claims based on the requests from signal providers.
      * Each actuator can be claimed by only one provider at any given time.


### FR5-AD
    Title: Command transmission capabilities of Databroker to Provider

    Status: 🟢/🟡

    Description: The Databroker shall be able to send to the Provider the following commands
    * Actuate on actuator. 🟢
    * Start receiving signal values from the Provider. 🟡
    * Stop receiving signal values from the Provider. 🟡

## As Provider
### FR1-AP
    Title: Provider actuator ownership claim management by Databroker

    Status: 🟢

    Description: The Databroker shall offer a method to Providers allowing to claim providership of a set of actuators.
      * IF all claimed actuators are known AND
            the provider has providing rights for all claimed actuators AND
            all claimed actuators are NOT yet claimed by another provider
          THEN the Databroker shall accept and store the claim
        ELSE the Databroker shall reject the overall claim and return an error containing the reason.

      * The Databroker shall remember accepted claims of a provider if the connection to the provider is lost.
      * The Databroker shall allow providers to re-claim previously successfully claimed actuators.

### FR2-AP
    Title: High-Frequency publishing of Signal Values by Provider

    Status: 🟢

    Description: The Databroker shall be capable of publishing signal values at the cycle time received from the Vehicle Network.
      * IF all published signal are known AND
            the provider has providing rights for all signals
          THEN the Databroker shall accept and store the values
          ELSE the Databroker shall reject the overall request and return an error response containing the reason.

### FR3-AP
    Title: Publishing of Signal Values by Provider

    Status: 🟢

    Description: The Databroker shall be able to publish signal values.
      * IF all published signal are known AND
            the provider has providing rights for all signals
          THEN the Databroker shall accept and store the values
          ELSE the Databroker shall reject the overall request and return an error response containing the reason.

### FR4-AP
    Title: Actuation notification handling

    Status: 🟢

    Description: The Databroker shall offer a method to providers to subscribe for actuation notifications on a (sub-) set of claimed actuators.
      * Provider shall only receive actuation requests for actuators that it owns or manages.
      * Provider shall process actuation requests and forward them to the Vehicle Network.
      * Provider shall notify the Databroker back right after the actuation request was forwarded.

### FR5-AP
    Title: Signal state update mechanism for providers by Databroker

    Status: 🔴

    Description: The Databroker shall offer a method to providers to update the current state of a set of signals.
      a. The current state consists a timestamp and either of valid value or a failure state.
      b. The Databroker shall offer a method optimized for frequent updates.
      c. The Databroker should offer a second method for non-frequent updates that is easy to use in a provider's implementation.
      d. The Databroker shall reject updating the current state of signals where the client is not the provider of.
      e. The Databroker shall store the updated value or failure state of a signal.

# Non-Functional Requirements

# Domain requirements
