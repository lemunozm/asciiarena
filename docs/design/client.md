# Client

## Architecture

<p align="center">
  <img src="https://docs.google.com/drawings/d/e/2PACX-1vTzsUS1y8meOBUVh54lM8ylcLvgFUwQ2V1Fy2-oQ1v-vyfOE6IoQHp_aaA0T7vSn2kAslHxqQC6HYoT/pub?w=840&h=507" width"840"/>
</p>

The left side of applications represents the modules that receive input from the outside.
This input is processed and sent to the application.
The application will dispatch the events synchronously to the right side of modules.
These modules will process this input, and together with the state, generate the output.

The right side of the application follows the Model View Controller pattern.
 - The model is represented by the state module.
 - The controller is management by the actions located in the store module.
 - The view is shown by the renderer module.

### Use case Example

#### The user press `enter` key to connect to the server

The **input** module detect the enter key pressed as an event and
send it to the **application** module that enqueue this event to be processed in the correct order.
From the main thread, the **application** module read the input event and
send it to the **gui** module in order to be processed.

This **gui** module will check some conditions (some of them reading the **state**):
correct panel? correct focus? correct server address value? and
if the conditions are satisfied then it generates an action
that is dispatched by the **store module**.

Then the **store** mutates some content of the **state** module and
make api call event that the **server proxy module** will receive and process into its own thread.

Once the input has been processed, a drawing event is dispatched when
the frame needs to be rerendered. The **renderer** module reads the modified **state**
and draw a frame according to its content.

Asynchronously, the **server proxy** will generates an event with
the response about the connection attempt.
Similar to the input module, the event will be enqueued by the **application** and
reading by the main thread that will use the **store** to dispatch the action.
Then, the **store** will modify the **state** with the connection result.

By last, in the next drawing event, the **renderer** will draw a new frame based in the new
**state** (with the connection result).
