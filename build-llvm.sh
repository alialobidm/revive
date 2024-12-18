#!/usr/bin/env bash

set -euo pipefail

INSTALL_DIR="${PWD}/llvm18.0"
mkdir -p ${INSTALL_DIR}

# Build LLVM, clang
LLVM_SRC_PREFIX=${PWD}/llvm-project
LLVM_SRC_DIR=${LLVM_SRC_PREFIX}/llvm
LLVM_BUILD_DIR=${PWD}/build/llvm

./clone-llvm.sh "${LLVM_SRC_PREFIX}"

if [ ! -d ${LLVM_BUILD_DIR} ] ; then
	mkdir -p ${LLVM_BUILD_DIR}
fi

cmake -G Ninja \
  -S ${LLVM_SRC_DIR} \
  -B ${LLVM_BUILD_DIR} \
  -DLLVM_ENABLE_ASSERTIONS=On \
  -DLLVM_ENABLE_TERMINFO=Off \
  -DLLVM_ENABLE_LIBXML2=Off \
  -DLLVM_ENABLE_ZLIB=Off \
  -DLLVM_ENABLE_PROJECTS='clang;lld' \
  -DLLVM_TARGETS_TO_BUILD='RISCV' \
  -DLLVM_ENABLE_ZSTD=Off \
  -DCMAKE_BUILD_TYPE=MinSizeRel \
  -DCMAKE_INSTALL_PREFIX=${INSTALL_DIR}

cmake --build ${LLVM_BUILD_DIR}
cmake --install ${LLVM_BUILD_DIR}

# Build compiler builtins
COMPILER_RT_SRC_DIR=${LLVM_SRC_PREFIX}/compiler-rt
COMPILER_RT_BUILD_DIR=${PWD}/build/compiler-rt
if [ ! -d ${COMPILER_RT_BUILD_DIR} ] ; then
	mkdir -p ${COMPILER_RT_BUILD_DIR}
fi

build_compiler_rt() {
	case "$1" in
		64) TARGET_ABI=lp64e ;;
		32) TARGET_ABI=ilp32e ;;
		*) exit -1
	esac
	CFLAGS="--target=riscv${1} -march=rv${1}em -mabi=${TARGET_ABI} -mcpu=generic-rv${1} -nostdlib -nodefaultlibs"

	cmake -G Ninja \
	  -S ${COMPILER_RT_SRC_DIR} \
	  -B ${COMPILER_RT_BUILD_DIR} \
	  -DCMAKE_BUILD_TYPE=Release \
	  -DCMAKE_INSTALL_PREFIX=${INSTALL_DIR} \
	  -DCOMPILER_RT_BUILD_BUILTINS=ON \
	  -DCOMPILER_RT_BUILD_LIBFUZZER=OFF \
	  -DCOMPILER_RT_BUILD_MEMPROF=OFF \
	  -DCOMPILER_RT_BUILD_PROFILE=OFF \
	  -DCOMPILER_RT_BUILD_SANITIZERS=OFF \
	  -DCOMPILER_RT_BUILD_XRAY=OFF \
	  -DCMAKE_C_COMPILER=${INSTALL_DIR}/bin/clang \
	  -DCMAKE_C_COMPILER_TARGET=riscv${1} \
	  -DCMAKE_ASM_COMPILER_TARGET=riscv${1} \
	  -DCMAKE_CXX_COMPILER_TARGET=riscv${1} \
	  -DCMAKE_C_TARGET_BITS=riscv${1} \
	  -DCMAKE_ASM_TARGET_BITS=riscv${1} \
	  -DCMAKE_AR=${INSTALL_DIR}/bin/llvm-ar \
	  -DCMAKE_NM=${INSTALL_DIR}/bin/llvm-nm \
	  -DCMAKE_RANLIB=${INSTALL_DIR}/bin/llvm-ranlib \
	  -DCOMPILER_RT_BAREMETAL_BUILD=ON \
	  -DLLVM_CONFIG_PATH=${INSTALL_DIR}/bin/llvm-config \
	  -DCMAKE_C_FLAGS="${CFLAGS}" \
	  -DCMAKE_ASM_FLAGS="${CFLAGS}" \
	  -DCOMPILER_RT_TEST_COMPILER=${INSTALL_DIR}/bin/clang \
	  -DCMAKE_CXX_FLAGS="${CFLAGS}" \
	  -DCMAKE_SYSTEM_NAME=unknown \
	  -DCOMPILER_RT_DEFAULT_TARGET_ONLY=ON 
	
	cmake --build ${COMPILER_RT_BUILD_DIR}
	cmake --install ${COMPILER_RT_BUILD_DIR} 
}

build_compiler_rt 32
build_compiler_rt 64

echo ""
echo "success"
echo "add this directory to your PATH: ${INSTALL_DIR}/bin/"
