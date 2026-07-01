/*
 * Copyright 2026 mcsimon
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include "ipc/parcel.h"
#include <stdio.h>
#include <assert.h>
#include <string.h>

int main(void) {
    printf("\n🧪 Running IPC Serialization Unit Tests...\n");

    parcel_t p;
    parcel_init(&p);

    // 1. Test Writing Primitives
    assert(parcel_write_int32(&p, 42) == 0);
    assert(parcel_write_uint32(&p, 1337) == 0);
    assert(parcel_write_string(&p, "Hello Orion OS!") == 0);

    // 2. Test Reading Primitives
    int32_t val1;
    uint32_t val2;
    char str_buf[64];

    assert(parcel_read_int32(&p, &val1) == 0);
    assert(val1 == 42);
    printf("  ✅ Int32 Match: %d == 42\n", val1);

    assert(parcel_read_uint32(&p, &val2) == 0);
    assert(val2 == 1337);
    printf("  ✅ UInt32 Match: %u == 1337\n", val2);

    assert(parcel_read_string(&p, str_buf, sizeof(str_buf)) == 0);
    assert(strcmp(str_buf, "Hello Orion OS!") == 0);
    printf("  ✅ String Match: \"%s\" == \"Hello Orion OS!\"\n", str_buf);

    parcel_free(&p);
    printf("🎉 All IPC Unit Tests Passed Successfully!\n\n");
    return 0;
}
