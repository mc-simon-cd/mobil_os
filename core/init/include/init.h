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

#ifndef INIT_H
#define INIT_H

#define _GNU_SOURCE  // Expose strdup and modern POSIX interfaces in standard headers

#include <stdio.h>
#include <sys/types.h>

#define MAX_SERVICES 32
#define MAX_PROPERTIES 128
#define MAX_ARGS 16

// Custom Logging levels and macros
#define LOG_TAG "INIT"
#define LOG_INFO(fmt, ...)  fprintf(stdout, "[INFO] [" LOG_TAG "] " fmt "\n", ##__VA_ARGS__)
#define LOG_WARN(fmt, ...)  fprintf(stderr, "[WARN] [" LOG_TAG "] " fmt "\n", ##__VA_ARGS__)
#define LOG_ERROR(fmt, ...) fprintf(stderr, "[ERR]  [" LOG_TAG "] " fmt "\n", ##__VA_ARGS__)

typedef enum {
    SVC_CLASS_DEFAULT = 0,
    SVC_CLASS_CORE,
    SVC_CLASS_MAIN
} service_class_t;

// Service structure representing daemons to launch/manage
typedef struct {
    char name[64];
    char path[128];
    char *args[MAX_ARGS];
    int arg_count;
    pid_t pid;
    int respawn;      // Boolean flag to auto-restart on exit
    int active;       // Status track flag
    service_class_t service_class;
} service_t;

// System dynamic properties
typedef struct {
    char key[64];
    char value[128];
} property_t;

// Function declarations - Mounting
int mount_essential_filesystems(void);

// Function declarations - Properties
int property_init(void);
int property_set(const char *key, const char *value);
const char* property_get(const char *key);

// Function declarations - Parser & Runner
int parse_init_rc(const char *filename);
int launch_services(void);
void handle_sigchld(int sig);
service_t* get_service_by_pid(pid_t pid);
void respawn_service(service_t *svc);

#endif // INIT_H
