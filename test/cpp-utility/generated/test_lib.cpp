
#include "test_lib.h"

#ifdef _MSC_VER
    const char test_lib::Descriptor::NAME[] = "test_lib.dll";
#else
    const char test_lib::Descriptor::NAME[] = "libtest_lib.so";
#endif

const char test_lib::Descriptor::WINDOWS_NAME[] = "test_lib.dll";
const char test_lib::Descriptor::POSIX_NAME[] = "libtest_lib.so";

const std::array< intercom::CLSID, 11 > test_lib::Descriptor::CLASSES = { {

    intercom::CLSID{0xf06af5f0,0x745a,0x3b29,{0x48,0x39,0xd2,0xd3,0x9a,0x3f,0x08,0xcd}},
    intercom::CLSID{0x12341234,0x1234,0x1234,{0x12,0x34,0x12,0x34,0x12,0x34,0x00,0x01}},
    intercom::CLSID{0x694c1893,0x2fa8,0x3d4c,{0x6a,0xcf,0x69,0xc5,0x93,0x66,0x72,0x1e}},
    intercom::CLSID{0xe5ce34c4,0xc1ad,0x34bc,{0x69,0xf6,0xd1,0xbf,0xa6,0xbb,0x25,0x96}},
    intercom::CLSID{0x3323cccd,0x1a38,0x33a4,{0x4a,0xe1,0x4d,0xc9,0x2a,0x7e,0x8d,0xc5}},
    intercom::CLSID{0x51ed4fb8,0x35d8,0x36c6,{0x78,0xfd,0x6b,0xc5,0x82,0xc8,0x4b,0x19}},
    intercom::CLSID{0x88687644,0x9cb2,0x3bd6,{0x4c,0x23,0xdb,0x54,0x7d,0x39,0x90,0x29}},
    intercom::CLSID{0x2af563c2,0xdc1c,0x339a,{0x60,0x35,0xf5,0xf8,0x18,0x0f,0xae,0x86}},
    intercom::CLSID{0xf2a61371,0xf376,0x3437,{0x70,0xb7,0x46,0x5c,0x76,0x09,0x11,0x3a}},
    intercom::CLSID{0x502e111f,0xb49b,0x3e02,{0x4e,0x5f,0x97,0x1b,0xdd,0xef,0x24,0xaf}},
    intercom::CLSID{0xdf3c35c1,0xcdd2,0x3b15,{0x6a,0x24,0xa7,0xe9,0xd6,0xb3,0xdd,0xf0}}
} };

const intercom::IID test_lib::raw::IRefCount::ID = {0xaa5b7352,0x5d7a,0x35b9,{0x52,0x06,0x14,0x5b,0x04,0x1f,0x2c,0x1c}};
const intercom::IID test_lib::raw::IPrimitiveOperations::ID = {0x12341234,0x1234,0x1234,{0x12,0x34,0x12,0x34,0x12,0x34,0x00,0x02}};
const intercom::IID test_lib::raw::IStatefulOperations::ID = {0x2b9bddd2,0x31f5,0x3d4b,{0x79,0xa0,0xac,0x8e,0x8d,0x11,0xa9,0x3e}};
const intercom::IID test_lib::raw::IResultOperations::ID = {0xffb673d9,0x7896,0x3a4c,{0x4f,0xa8,0xf7,0x24,0x06,0x58,0x8a,0xa1}};
const intercom::IID test_lib::raw::IClassCreator::ID = {0x2e7e23e8,0xf66d,0x3779,{0x6c,0x74,0x61,0x89,0x8d,0x7b,0x40,0xcd}};
const intercom::IID test_lib::raw::ICreatedClass::ID = {0x104eb174,0xfd00,0x3ecf,{0x7e,0x0d,0xd9,0x65,0x56,0x42,0x79,0xe7}};
const intercom::IID test_lib::raw::IParent::ID = {0xcea1c199,0xbf71,0x3b0a,{0x5a,0x4c,0xee,0x3f,0x5a,0x0a,0xe5,0xce}};
const intercom::IID test_lib::raw::IRefCountOperations::ID = {0x6b198a07,0x2d86,0x340e,{0x71,0x7e,0xf4,0x16,0xa3,0x90,0x5d,0x6e}};
const intercom::IID test_lib::raw::ISharedInterface::ID = {0x1df08ff6,0xaafb,0x37ec,{0x53,0xcf,0xcd,0xe2,0x49,0xce,0xeb,0x4b}};
const intercom::IID test_lib::raw::IErrorSource::ID = {0x5505b7b6,0x5ca4,0x3e38,{0x66,0x7b,0xa9,0x82,0x3f,0x1d,0x5a,0x0f}};
const intercom::IID test_lib::raw::IAllocTests::ID = {0xb5c84f5f,0xb69f,0x3071,{0x7a,0xd6,0x0f,0xea,0x72,0xe8,0x95,0xc4}};
const intercom::IID test_lib::raw::IStringTests::ID = {0x936befdc,0x4239,0x3464,{0x63,0x79,0x0a,0xbd,0xa7,0x22,0xa2,0x5b}};
const intercom::IID test_lib::raw::IAllocator::ID = {0x18ee22b3,0xb0c6,0x44a5,{0xa9,0x4a,0x7a,0x41,0x76,0x76,0xfb,0x66}};


const intercom::CLSID test_lib::raw::RefCountOperationsDescriptor::ID = {0xf06af5f0,0x745a,0x3b29,{0x48,0x39,0xd2,0xd3,0x9a,0x3f,0x08,0xcd}};
const std::array<intercom::IID, 1> test_lib::raw::RefCountOperationsDescriptor::INTERFACES = { {
             test_lib::raw::IRefCountOperations::ID
} };

const intercom::CLSID test_lib::raw::PrimitiveOperationsDescriptor::ID = {0x12341234,0x1234,0x1234,{0x12,0x34,0x12,0x34,0x12,0x34,0x00,0x01}};
const std::array<intercom::IID, 1> test_lib::raw::PrimitiveOperationsDescriptor::INTERFACES = { {
             test_lib::raw::IPrimitiveOperations::ID
} };

const intercom::CLSID test_lib::raw::StatefulOperationsDescriptor::ID = {0x694c1893,0x2fa8,0x3d4c,{0x6a,0xcf,0x69,0xc5,0x93,0x66,0x72,0x1e}};
const std::array<intercom::IID, 1> test_lib::raw::StatefulOperationsDescriptor::INTERFACES = { {
             test_lib::raw::IStatefulOperations::ID
} };

const intercom::CLSID test_lib::raw::ResultOperationsDescriptor::ID = {0xe5ce34c4,0xc1ad,0x34bc,{0x69,0xf6,0xd1,0xbf,0xa6,0xbb,0x25,0x96}};
const std::array<intercom::IID, 1> test_lib::raw::ResultOperationsDescriptor::INTERFACES = { {
             test_lib::raw::IResultOperations::ID
} };

const intercom::CLSID test_lib::raw::ClassCreatorDescriptor::ID = {0x3323cccd,0x1a38,0x33a4,{0x4a,0xe1,0x4d,0xc9,0x2a,0x7e,0x8d,0xc5}};
const std::array<intercom::IID, 1> test_lib::raw::ClassCreatorDescriptor::INTERFACES = { {
             test_lib::raw::IClassCreator::ID
} };

const intercom::CLSID test_lib::raw::CreatedClassDescriptor::ID = {0x51ed4fb8,0x35d8,0x36c6,{0x78,0xfd,0x6b,0xc5,0x82,0xc8,0x4b,0x19}};
const std::array<intercom::IID, 3> test_lib::raw::CreatedClassDescriptor::INTERFACES = { {
             test_lib::raw::ICreatedClass::ID,
             test_lib::raw::IParent::ID,
             test_lib::raw::IRefCount::ID
} };

const intercom::CLSID test_lib::raw::SharedImplementationDescriptor::ID = {0x88687644,0x9cb2,0x3bd6,{0x4c,0x23,0xdb,0x54,0x7d,0x39,0x90,0x29}};
const std::array<intercom::IID, 1> test_lib::raw::SharedImplementationDescriptor::INTERFACES = { {
             test_lib::raw::ISharedInterface::ID
} };

const intercom::CLSID test_lib::raw::ErrorSourceDescriptor::ID = {0x2af563c2,0xdc1c,0x339a,{0x60,0x35,0xf5,0xf8,0x18,0x0f,0xae,0x86}};
const std::array<intercom::IID, 1> test_lib::raw::ErrorSourceDescriptor::INTERFACES = { {
             test_lib::raw::IErrorSource::ID
} };

const intercom::CLSID test_lib::raw::AllocTestsDescriptor::ID = {0xf2a61371,0xf376,0x3437,{0x70,0xb7,0x46,0x5c,0x76,0x09,0x11,0x3a}};
const std::array<intercom::IID, 1> test_lib::raw::AllocTestsDescriptor::INTERFACES = { {
             test_lib::raw::IAllocTests::ID
} };

const intercom::CLSID test_lib::raw::StringTestsDescriptor::ID = {0x502e111f,0xb49b,0x3e02,{0x4e,0x5f,0x97,0x1b,0xdd,0xef,0x24,0xaf}};
const std::array<intercom::IID, 1> test_lib::raw::StringTestsDescriptor::INTERFACES = { {
             test_lib::raw::IStringTests::ID
} };

const intercom::CLSID test_lib::raw::AllocatorDescriptor::ID = {0xdf3c35c1,0xcdd2,0x3b15,{0x6a,0x24,0xa7,0xe9,0xd6,0xb3,0xdd,0xf0}};
const std::array<intercom::IID, 1> test_lib::raw::AllocatorDescriptor::INTERFACES = { {
             test_lib::raw::IAllocator::ID
} };
