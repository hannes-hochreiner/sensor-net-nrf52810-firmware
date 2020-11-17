SensorNet nRF52810 firmware
===========================

Firmware for SensorNet gateway and sensors based on the nRF52810.

`SensorNet Gateway BL651 <https://github.com/hannes-hochreiner/sensor-net-gateway-bl651>`_
------------------------------------------------------------------------------------------

The messages from the gateway are formatted as JSON with a terminating line feed.

.. code-block:: JSON

  {
    "type": "gateway-bl651-sensor",
    "message": {
      "mcuId": "<string>",
      "index": "<integer>",
      "sensorId": "<string>",
      "temperature": "<float>",
      "humidity": "<float>"
    }
  }

.. code-block:: JSON

  {
    "type": "gateway-bl651-radio",
    "rssi": "<integer (dB)>",
    "data": "<string (hex encoded binary data)>"
  }

Personal Beacon
---------------

The fields of the payload of the packets are defined in the table below.
All values are transmitted in little-endian.

+----------------+----+-----------------------+
|name            |type|value                  |
+================+====+=======================+
|type            |u8  |"3"                    |
+----------------+----+-----------------------+
|device id       |u64 |MCU id                 |
+----------------+----+-----------------------+
|part id         |u32 |MCU model              |
+----------------+----+-----------------------+
|index           |u32 |running count          |
+----------------+----+-----------------------+
|sensor id       |u16 |self-assigned sensor id|
+----------------+----+-----------------------+
|acceleration x  |i16 |arbitrary units        |
+----------------+----+-----------------------+
|acceleration y  |i16 |arbitrary units        |
+----------------+----+-----------------------+
|acceleration z  |i16 |arbitrary units        |
+----------------+----+-----------------------+
|magnetic field x|i16 |arbitrary units        |
+----------------+----+-----------------------+
|magnetic field y|i16 |arbitrary units        |
+----------------+----+-----------------------+
|magnetic field z|i16 |arbitrary units        |
+----------------+----+-----------------------+

Power Consumption
.................

+------------+---------+---------+-----------+
|phase       |duration |component|consumption|
+============+=========+=========+===========+
|sleep       |59,9895ms|MCU      | 0.0015mA  |
|            |         +---------+-----------+ 
|            |         |Sensor   | 0.0020mA  |
|            |         +---------+-----------+ 
|            |         |sub-total| 0.0035mA  |
+------------+---------+---------+-----------+
|measurement |10ms     |MCU      | 2.2000mA  |
|            |         +---------+-----------+ 
|            |         |Sensor   | 0.2500mA  |
|            |         +---------+-----------+ 
|            |         |sub-total| 2.4500mA  |
+------------+---------+---------+-----------+
|transmission|0.5ms    |MCU      | 2.2000mA  |
|            |         +---------+-----------+ 
|            |         |Sensor   | 0.0020mA  |
|            |         +---------+-----------+ 
|            |         |Radio    | 8.0000mA  |
|            |         +---------+-----------+ 
|            |         |sub-total|10.2020mA  |
+------------+---------+---------+-----------+

On average: 35.5µA (about 230 days on a 200mAh battery)

Energy Saving
-------------

  * Enabling the DC/DC converter halves the current requirement in some scenarios.
  * Power radio off, when it is not in use (it is on after reset)
  * Running from HFINT consumes about 10% less energy than running from HFXO
  * Running RTC from LFXO saves about 30% in sleep as compared to using LFRC
