/* ea:   0x00044004 */
/* name: sub_44004 */
/* size: 1016 */
void sub_44004(_DWORD *lpFileName, ...)
{
  HANDLE FileW; // r7
  int v3; // r1
  int v4; // r2
  void *v5; // r0
  int v6; // r1
  int v7; // r2
  int v8; // r4
  int v9; // r1
  int v10; // r2
  _DWORD *v11; // r2
  _DWORD *v12; // r4
  _DWORD *v13; // r4
  unsigned int i; // r5
  void *v15; // r0
  int v16; // r1
  int v17; // r2
  int v18; // r0
  _DWORD *v19; // r1
  _DWORD *v20; // r4
  void *v21; // r0
  int v22; // r1
  int v23; // r2
  int v24; // r0
  _DWORD *v25; // r1
  _DWORD *v26; // r4
  DWORD NumberOfBytesRead; // [sp+Ch] [bp-80h] BYREF
  unsigned int v28; // [sp+10h] [bp-7Ch] BYREF
  unsigned int Buffer; // [sp+14h] [bp-78h] BYREF
  int v30; // [sp+18h] [bp-74h]
  int v31; // [sp+1Ch] [bp-70h]
  int v32; // [sp+20h] [bp-6Ch]
  int v33; // [sp+24h] [bp-68h]
  int v34; // [sp+28h] [bp-64h]
  _DWORD v35[20]; // [sp+2Ch] [bp-60h] BYREF
  void *varg_r1; // [sp+80h] [bp-Ch] BYREF
  va_list varg_r1a; // [sp+80h] [bp-Ch]
  int varg_r2; // [sp+84h] [bp-8h]
  int varg_r3; // [sp+88h] [bp-4h]
  LPCWSTR lpFileNamea; // [sp+8Ch] [bp+0h] BYREF
  va_list lpFileNameb; // [sp+8Ch] [bp+0h]
  va_list va2; // [sp+90h] [bp+4h] BYREF

  va_start(va2, lpFileName);
  va_start(lpFileNameb, lpFileName);
  va_start(varg_r1a, lpFileName);
  varg_r1 = va_arg(lpFileNameb, void *);
  varg_r2 = va_arg(lpFileNameb, _DWORD);
  varg_r3 = va_arg(lpFileNameb, _DWORD);
  va_copy(va2, lpFileNameb);
  lpFileNamea = va_arg(va2, LPCWSTR);
  FileW = CreateFileW(lpFileNamea, 0x80000000, 1u, nullptr, 3u, 0x80u, nullptr);
  ReadFile(FileW, lpFileName + 245, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(FileW, lpFileName + 246, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(FileW, &Buffer, 0x18u, &NumberOfBytesRead, nullptr);
  sub_45EC0(v35, v3, v4);
  if ( v30 != -1 )
  {
    v5 = operator new(0x34u);
    if ( v5 != nullptr )
      v8 = sub_45EC0(v5, v6, v7);
    else
      v8 = 0;
    v9 = v31;
    v10 = v32;
    lpFileName[247] = v8;
    sub_46294(v8, v9, v10);
    *(_DWORD *)(lpFileName[247] + 4) = Buffer;
    *(_DWORD *)(lpFileName[247] + 8) = v30;
    *(_DWORD *)(lpFileName[247] + 24) = 0;
    v11 = (_DWORD *)lpFileName[247];
    if ( *v11 == 0 )
      __debugbreak();
    v11[9] = *(_DWORD *)(*v11 + 4);
    v12 = (_DWORD *)lpFileName[247];
    if ( *v12 == 0 )
      __debugbreak();
    v12[10] = *(_DWORD *)(*v12 + 8) / v12[1];
    v13 = (_DWORD *)lpFileName[247];
    if ( *v13 == 0 )
      __debugbreak();
    lpFileName[248] = lpFileName[245] / *(_DWORD *)(*v13 + 4) + 1;
    if ( *v13 == 0 )
      __debugbreak();
    lpFileName[249] = lpFileName[246] / *(_DWORD *)(*v13 + 8) * v13[1] + 1;
  }
  ReadFile(FileW, &v28, 4u, &NumberOfBytesRead, nullptr);
  for ( i = 0; i < v28; ++i )
  {
    ReadFile(FileW, &Buffer, 0x18u, &NumberOfBytesRead, nullptr);
    sub_46294(v35, v31, v32);
    v35[6] = v34;
    v35[1] = Buffer;
    v35[2] = v30;
    v35[7] = v33;
    if ( v35[0] == 0 )
      __debugbreak();
    v35[9] = *(_DWORD *)(v35[0] + 4);
    v35[10] = *(_DWORD *)(v35[0] + 8) / Buffer;
    sub_449A8(lpFileName + 244, v35);
  }
  v15 = operator new(0x34u);
  if ( v15 != nullptr )
    v18 = sub_45EC0(v15, v16, v17);
  else
    v18 = 0;
  lpFileName[250] = v18;
  sub_46294(v18, 0, 0);
  *(_DWORD *)(lpFileName[250] + 24) = 0;
  *(_DWORD *)(lpFileName[250] + 4) = 1;
  *(_DWORD *)(lpFileName[250] + 8) = 21;
  *(_DWORD *)(lpFileName[250] + 28) = 1;
  v19 = (_DWORD *)lpFileName[250];
  if ( *v19 == 0 )
    __debugbreak();
  v19[9] = *(_DWORD *)(*v19 + 4);
  v20 = (_DWORD *)lpFileName[250];
  if ( *v20 == 0 )
    __debugbreak();
  v20[10] = *(_DWORD *)(*v20 + 8) / v20[1];
  v21 = operator new(0x34u);
  if ( v21 != nullptr )
    v24 = sub_45EC0(v21, v22, v23);
  else
    v24 = 0;
  lpFileName[251] = v24;
  sub_46294(v24, 0, 0);
  *(_DWORD *)(lpFileName[251] + 24) = 0;
  *(_DWORD *)(lpFileName[251] + 4) = 1;
  *(_DWORD *)(lpFileName[251] + 8) = 23;
  *(_DWORD *)(lpFileName[251] + 28) = 1;
  v25 = (_DWORD *)lpFileName[251];
  if ( *v25 == 0 )
    __debugbreak();
  v25[9] = *(_DWORD *)(*v25 + 4);
  v26 = (_DWORD *)lpFileName[251];
  if ( *v26 == 0 )
    __debugbreak();
  v26[10] = *(_DWORD *)(*v26 + 8) / v26[1];
  CloseHandle(FileW);
  sub_45EFC(v35);
  sub_12F48((void **)varg_r1a);
  sub_12F48((void **)lpFileNameb);
}

