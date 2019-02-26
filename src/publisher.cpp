#include <srm/core.h>
#include <srm/node.h>

static const SrmNodeVtbl vtbl = {
    
};

class Publisher {
public:
    explicit Publisher(SrmCore core) noexcept : core_(core) { }

private:
    SrmCore core_;
};

SRM_SHARED_OBJECT_EXPORT const SrmNodeVtbl* srm_Node_get_vtbl(void) {

}
