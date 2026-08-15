// esp32-h21_sensor/main/main.c

#include <stdio.h>
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_log.h"
#include <string.h>
#include <math.h>

// Mocks for missing hardware headers to allow compilation in standard CI
typedef struct { int dummy; } esp_zb_platform_config_t;
#define ESP_ZB_PLATFORM_CONFIG_DEFAULT() {0}
static void esp_zb_platform_config(esp_zb_platform_config_t *cfg) {}
static void esp_zb_zcl_send_report() {}

static float dht11_read_temperature(int pin) { return 25.0f; }
static float dht11_read_humidity(int pin) { return 50.0f; }
static float ldr_read_light(int pin) { return 1000.0f; }
static void dht11_init(int pin) {}
static void ldr_init(int pin) {}

//#include "esp_zigbee_core.h"
//#include "esp_zb_signal.h"

#include "dht11.h"   // Sensor de temperatura/humidade
#include "ldr.h"     // Sensor de luz

#define SENSOR_PIN  GPIO_NUM_4
#define LDR_PIN     GPIO_NUM_5

static const char *TAG = "VAJRA_NANO";

void collect_entropy(float *temp, float *hum, float *lux) {
    *temp = dht11_read_temperature(SENSOR_PIN);
    *hum  = dht11_read_humidity(SENSOR_PIN);
    *lux  = ldr_read_light(LDR_PIN);
}

void zigbee_send_entropy(float temp, float hum, float lux) {
    // Montar payload com métricas de entropia
    uint8_t payload[12];
    memcpy(payload, (uint8_t*)&temp, 4);
    memcpy(payload+4, (uint8_t*)&hum, 4);
    memcpy(payload+8, (uint8_t*)&lux, 4);

    // Enviar via Zigbee para o ESP32‑C6 (coordenador)
    esp_zb_zcl_send_report();
}

void app_main() {
    esp_zb_platform_config_t config = ESP_ZB_PLATFORM_CONFIG_DEFAULT();
    esp_zb_platform_config(&config);

    dht11_init(SENSOR_PIN);
    ldr_init(LDR_PIN);

    while (1) {
        float temp, hum, lux;
        collect_entropy(&temp, &hum, &lux);

        // Calcular entropia simples (proxy)
        float entropy = -logf(temp / 100.0f) - logf(hum / 100.0f) - logf(lux / 1000.0f);
        ESP_LOGI(TAG, "Entropia local: %.4f (T=%.1f°C, H=%.1f%%, L=%.1f lux)", entropy, temp, hum, lux);

        // Enviar para o coordenador Zigbee (ESP32‑C6)
        zigbee_send_entropy(temp, hum, lux);

        vTaskDelay(pdMS_TO_TICKS(5000));
    }
}
