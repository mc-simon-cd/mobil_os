#ifndef I18N_H
#define I18N_H

// Initializes the i18n localization system.
// Loads translation file from "<locale_dir>/<locale_name>.txt".
// Returns 0 on success, or -1 on error.
int i18n_init(const char *locale_dir, const char *locale_name);

// Returns the localized string for the given key.
// If the key is not found, returns the key itself as a fallback.
const char *i18n_get(const char *key);

// Frees all loaded translation memory resources.
void i18n_free(void);

#endif // I18N_H
