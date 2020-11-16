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

|----------------+----+-----------------------+
|name            |type|value                  |
|================+====+=======================+
|type            |u8  |"3"                    |
|----------------+----+-----------------------+
|device id       |u64 |MCU id                 |
|----------------+----+-----------------------+
|part id         |u32 |MCU model              |
|----------------+----+-----------------------+
|index           |u32 |running count          |
|----------------+----+-----------------------+
|sensor id       |u16 |self-assigned sensor id|
|----------------+----+-----------------------+
|acceleration x  |i16 |arbitrary units        |
|----------------+----+-----------------------+
|acceleration y  |i16 |arbitrary units        |
|----------------+----+-----------------------+
|acceleration z  |i16 |arbitrary units        |
|----------------+----+-----------------------+
|magnetic field x|i16 |arbitrary units        |
|----------------+----+-----------------------+
|magnetic field y|i16 |arbitrary units        |
|----------------+----+-----------------------+
|magnetic field z|i16 |arbitrary units        |
|----------------+----+-----------------------+
