# Client

## Architecture

<p align="center">
  <img src="https://docs.google.com/drawings/d/e/2PACX-1vTzsUS1y8meOBUVh54lM8ylcLvgFUwQ2V1Fy2-oQ1v-vyfOE6IoQHp_aaA0T7vSn2kAslHxqQC6HYoT/pub?w=840&h=507" width"840"/>
</p>

The left side of applications represents the modules that receive input from the outside.
This input is processed and sending to the application.
The application will dispatch the events synchronized to the right side of modules.
These modules will process this input, and among with the state generate output

The right side of the application follows a Model View Controller pattern.
 - The model is represented by the state module.
 - The controller is management by the actions located in the store module.
 - The view is renderered by the renderer module.

