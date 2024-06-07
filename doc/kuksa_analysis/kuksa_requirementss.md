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
  - [As Provider](#as-provider)
    - [FR1-AP](#fr1-ap)
    - [FR2-AP](#fr2-ap)
    - [FR3-AP](#fr3-ap)
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
    Title: Single Access Service Point for Signal Consumer

    Status: 🟡

    Description: The Signal Consumer shall have a single service point accessible from my execution environment.
   
### FR2-ASC
    Title: Uniform Retrieval of Vehicle and Environmental Signal Values

    Status: 🟡

    Description: The Signal Consumer shall get sensor values or actuator values of the vehicle signals and its environment(air temperature, etc) in a uniform manner.
      1.  IF there is an error
          * THEN the Signal Consumer shall not receive any signal value, just one single error with detailed error information
          * ELSE the Signal Consumer shall receive exactly all the signals’ states(failure or value) requested.

          n. A signal can have value out of the set of the defined value restriction/data type but it can also have those additional failure states:
            - "unknown": The signal with the specified path/name is not defined on the system.
            - "access denied": The signal is basically known, but the consumer/provider has no permission to access it.
            - "not provided": The signal is basically known, but currently no provider is serving it or the provider lost the connection to the backend component serving this signal.
            - "invalid": The signal is being served by a provider, but currently has no valid value. This may happen if a specific value cannot be determined because of some (temporary) physical sensor error or a target value is currently not known, e.g. if a window is moved by a passenger via up/down button.
       
      2.  The Signal Consumer only shall get values for signal to which it has permission to.
      3.  The Signal Consumer shall provide the paths of the requested signals, which optionally may contain wildcards.
  
### FR3-ASC
    Title: Subscription and Notification for Signal Value Changes

    Status: 🟡

    Description: The Signal Consumer shall subscribe to sensor values or actuator current values of a signal and then get notified when some signal value changed.
      *  IF there is an error
          THEN the Signal Consumer shall not receive any signal value, just one single error with detailed error information
          ELSE the Signal Consumer shall receive exactly all the signals’ states(failure or value) requested.
      *  The Signal Consumer only shall get values for signal to which it has permission to.
      *  The Signal Consumer shall provide the paths of the requested signals.

### FR4-ASC
    Title: Filtered Interval Subscription for Signal Values

    Status: 🟡

    Description: The Signal Consumer shall subscribe and be able to apply a filter to receive a signal values with an interval of x milliseconds.
      *  IF there is an error
          THEN the Signal Consumer shall not receive any signal value, just one single error with detailed error information
          ELSE the Signal Consumer shall receive exactly all the signals’ states(failure or value) requested.
      *  The Signal Consumer only shall get values for signal to which it has permission to.
      *  The Signal Consumer shall provide the paths of the requested signals.

### FR5-ASC
    Title: Accessing Static Metadata of Signals

    Status: 🟡

    Description: A Signal Consumer shall be able to get static Metadata from signals.
    - Details:
      * All sensors, actuators, and attributes values for which a Signal Consumer has permission to.
      * The Signal Consumer shall provide the path, paths or wildcard of the signals.

### FR6-ASC
    Title: Retrieving Attribute Values for Signal Consumption

    Status: 🟡

    Description: A Signal Consumer shall be able to get attributes values.

### FR7-ASC
    Title: Actuation of Actuator Value with Databroker Confirmation

    Status: 🟡

    Description: The Signal Consumer shall be able to actuate a value of an actuator. This value has to be forwarded to the provider of this actuator if the provider is available, otherwise the target value is lost:
    - Details:
      * Databroker should not store the target value, just forward it to the provider.
    
      * IF there is no provider connected
          THEN Signal Consumer shall receive that there is no provider available.
          ELSE IF databroker forwarded correctly the value to the provider
            THEN Signal Consumer shall receive an acknowledgement of true receipt.
          ELSE Signal Consumer shall receive an acknowledgement of false receipt.

### FR8-ASC
    Title: Provider Availability Detection for Signal Consumer

    Status: 🔴

    Description: The Signal Consumer shall be able to know if there is a provider up and running.

## As Databroker
### FR1-AD
    Title: Handling of COVESA Vehicle Signal Specification (VSS) Syntax by Databroker

    Status: 🟡

    Description: The Databroker shall handle catalogs of signals described by the syntax as defined by the COVESA Vehicle Signal Specification (VSS). This relates to all aspects of the VSS syntax definition, which is also called VSS rule set. This implies that the data broker can handle the signal catalog as defined by the COVESA VSS.

### FR2-AD
    Title: Support for VSS Metadata Elements by Databroker

    Status: 🟡

    Description: The Databroker shall support at least those metadata elements as defined by the VSS rule set.

### FR3-AD
    Title: Subscription Management by Databroker

    Status: 🟡

    Description: The Databroker shall keep a local record of all subscriptions of signal consumers.
      - Detail:
          The Databroker shall add or remove subscriptions to a subscription pool according to the subscription requests of the Signal Consumer.

### FR4-AD
    Title: Command Transmission Capabilities of Databroker to Provider

    Status: 🟡

    Description: The Databroker shall be able to send to the Provider the following commands
    * Actuate on actuator.
    * Start receiving signal values from the Provider.
    * Stop receiving signal values from the Provider.

## As Provider
### FR1-AP
    Title: Provider Claim Management by Databroker

    Status: 🟡

    Description: The data broker shall offer a method to clients allowing to claim providership of a set of signals.
      a. IF all claimed signals are known AND
            the client has providing rights for all claimed signals AND
            all claimed signal is NOT yet claimed by another provider
          THEN the data broker shall accept and remember the claim
          ELSE the data broker shall reject the overall claim and return an error containing the reason.
      b. The data broker shall remember accepted claims of a provider if the connection to the provider is lost.
      c. The data broker shall allow providers to re-claim previously successfully claimed signals.
   
### FR2-AP
    Title: Subscription and Actuation Notification Handling for Providers by Databroker

    Status: 🟡

    Description: The data broker shall offer a method to providers to subscribe for actuation notifications on a (sub-) set of claimed signals.
      a. The data broker shall reject subscription requests containing signals where the client is not the provider of.
      b. The data broker shall notify the provider of a signal about received actuation requests.

### FR3-AP
    Title: Signal State Update Mechanism for Providers by Databroker

    Status: 🟡

    Description: The data broker shall offer a method to providers to update the current state of a set of signals.
      a. The current state consists a timestamp and either of valid value or a failure state.
      b. The data broker shall offer a method optimized for frequent updates.
      c. The data broker should offer a second method for non-frequent updates that is easy to use in a provider's implementation.
      d. The data broker shall reject updating the current state of signals where the client is not the provider of.
      e. The data broker shall store the updated value or failure state of a signal.

# Non-Functional Requirements


# Domain requirements

