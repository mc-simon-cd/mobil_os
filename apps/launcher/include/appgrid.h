#ifndef APPGRID_H
#define APPGRID_H

typedef struct {
    char name[32];
    char package[64];
    int icon_id;
} app_icon_t;

// Desktop App Grid methods
void appgrid_init(void);
void appgrid_render(void);

#endif // APPGRID_H
