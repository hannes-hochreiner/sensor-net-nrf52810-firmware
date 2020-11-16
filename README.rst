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
      "mcuId": <string>,
      "index": <integer>,
      "sensorId": <string>,
      "temperature": <float>,
      "humidity": <float>
    }
  }

.. code-block:: JSON

  {
    "type": "gateway-bl651-radio",
    "rssi": <integer (dB)>,
    "data": <string (hex encoded binary data)>
  }
