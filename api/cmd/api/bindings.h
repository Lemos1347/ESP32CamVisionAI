#include <stdint.h>
#include <stdlib.h>
#include <string.h>

typedef struct YOLOv8 YOLOv8;
YOLOv8 *load_model(const char *c_model_path, const char *c_save_dir);
const char *process_image(YOLOv8 *c_model, const uint8_t *buffer, int length);
void free_model(YOLOv8 *c_model);
void free_c_string(const char *s);
