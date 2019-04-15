#include <sensordata.h>
#include <cmath>

float SensorData::pressure_altitude_feet() const {
    return 145366.45 * (1.0 - std::pow((this->pressure/1013.25), 0.190284));
}

float SensorData::pressure_altitude() const {
    return pressure_altitude_feet() * 0.3048;
}
