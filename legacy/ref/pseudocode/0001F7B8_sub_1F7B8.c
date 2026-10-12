/* ea:   0x0001F7B8 */
/* name: sub_1F7B8 */
/* size: 2216 */
int __fastcall sub_1F7B8(_DWORD *a1, int a2, int a3, int a4)
{
  wchar_t *v8; // r4
  int v9; // r0
  __int16 **v10; // r0
  HANDLE FileW; // r0
  void *v12; // r8
  int v14; // r2
  _DWORD *v15; // r0
  int v16; // r0
  unsigned int i; // r4
  _DWORD *j; // r5
  void (__fastcall ***v19)(_DWORD, int); // r3
  _DWORD *k; // r5
  void *v21; // r0
  int v22; // r4
  _DWORD *v23; // r0
  _DWORD *v24; // r0
  _DWORD *v25; // r0
  int v26; // r0
  _DWORD *v27; // r0
  int v28; // r4
  bool v29; // zf
  _DWORD *v30; // r0
  int v31; // r0
  _DWORD *v32; // r0
  _DWORD *n; // r5
  _DWORD *ii; // r5
  _DWORD *jj; // r5
  int v36; // r3
  void (__fastcall ***v37)(_DWORD, int); // r3
  char v38; // [sp+Ch] [bp-A0h] BYREF
  char v39; // [sp+Dh] [bp-9Fh] BYREF
  __int16 v40; // [sp+Eh] [bp-9Eh] BYREF
  DWORD NumberOfBytesRead; // [sp+10h] [bp-9Ch] BYREF
  int v42; // [sp+14h] [bp-98h] BYREF
  int m; // [sp+18h] [bp-94h] BYREF
  int v44; // [sp+1Ch] [bp-90h] BYREF
  unsigned int v45; // [sp+20h] [bp-8Ch] BYREF
  int Buffer; // [sp+24h] [bp-88h] BYREF
  int v47; // [sp+28h] [bp-84h] BYREF
  int v48; // [sp+2Ch] [bp-80h] BYREF
  int v49; // [sp+30h] [bp-7Ch] BYREF
  int v50; // [sp+34h] [bp-78h] BYREF
  int v51; // [sp+38h] [bp-74h] BYREF
  int v52; // [sp+3Ch] [bp-70h] BYREF
  int v53; // [sp+40h] [bp-6Ch] BYREF
  LPCWSTR v54[2]; // [sp+44h] [bp-68h] BYREF
  LPCWSTR lpFileName[4]; // [sp+4Ch] [bp-60h] BYREF
  void *v56[4]; // [sp+5Ch] [bp-50h] BYREF
  _DWORD v57[4]; // [sp+6Ch] [bp-40h] BYREF
  void *v58[12]; // [sp+7Ch] [bp-30h] BYREF

  sub_1333C(v56);
  sub_1333C(lpFileName);
  sub_1309C((int)v56);
  switch ( a2 )
  {
    case 1:
      v8 = aDataGl1Sav;
      break;
    case 2:
      v8 = aDataGl2Sav;
      break;
    case 3:
      v8 = aDataGl3Sav;
      break;
    case 4:
      v8 = aDataGl4Sav;
      break;
    default:
      v8 = aDataGl0Sav;
      break;
  }
  v9 = sub_12F04(v8);
  sub_1318C((void **)lpFileName, (char *)v8, (int)&v8[v9]);
  v10 = (__int16 **)sub_37E2C(v58, v56, lpFileName);
  if ( v10 != (__int16 **)lpFileName )
    sub_17890((int)lpFileName, *v10, v10[1]);
  sub_12F48(v58);
  FileW = CreateFileW(lpFileName[0], 0x80000000, 0, nullptr, 3u, 0x80u, nullptr);
  v12 = FileW;
  if ( FileW == (HANDLE)-1 )
  {
    sub_12EA0((int)lpFileName);
    sub_12EA0((int)v56);
    return 0;
  }
  SetFilePointer(FileW, 2, nullptr, 0);
  Buffer = 0;
  ReadFile(v12, &Buffer, 4u, &NumberOfBytesRead, nullptr);
  if ( Buffer != 19810929 )
  {
    CloseHandle(v12);
    sub_12F48((void **)lpFileName);
    sub_12F48(v56);
    return 0;
  }
  v45 = 0;
  SetFilePointer(v12, 4, nullptr, 1u);
  ReadFile(v12, &dword_7FB70, 4u, &NumberOfBytesRead, nullptr);
  ReadFile(v12, &dword_7FB6C, 4u, &NumberOfBytesRead, nullptr);
  if ( (*(unsigned __int8 (__fastcall **)(_DWORD *, int, int))(*a1 + 4))(a1, a3, a4) == 0 )
  {
LABEL_32:
    sub_12F48((void **)lpFileName);
    sub_12F48(v56);
    return 0;
  }
  v14 = a1[36];
  a1[180] = 96;
  (*(void (__fastcall **)(_DWORD *, void *))(v14 + 4))(a1 + 36, v12);
  (*(void (__fastcall **)(_DWORD *, void *))(a1[18] + 4))(a1 + 18, v12);
  a1[31] = 0;
  sub_2F708(a1 + 25);
  a1[179] = 0;
  ReadFile(v12, a1 + 181, 4u, &NumberOfBytesRead, nullptr);
  (*(void (__fastcall **)(_DWORD *, void *))(a1[72] + 44))(a1 + 72, v12);
  ReadFile(v12, &v44, 4u, &NumberOfBytesRead, nullptr);
  v47 = 1;
  v15 = (_DWORD *)sub_2FA58(a1 + 133, &v47);
  *v15 = v44;
  ReadFile(v12, &v44, 4u, &NumberOfBytesRead, nullptr);
  v48 = 1;
  v16 = sub_2FA58(a1 + 133, &v48);
  *(_DWORD *)(v16 + 4) = v44;
  ReadFile(v12, &v45, 4u, &NumberOfBytesRead, nullptr);
  sub_30BD8(v57, &v38, &v39);
  for ( i = 0; i < v45; ++i )
  {
    ReadFile(v12, &v49, 4u, &NumberOfBytesRead, nullptr);
    *(_DWORD *)sub_1DD10(v57, &v49) = 0;
  }
  for ( j = *(_DWORD **)sub_30C2C(a1 + 124, &v52); j != (_DWORD *)*sub_1DE18(a1 + 124, &v52); j = sub_16A30(j) )
  {
    (*(void (__fastcall **)(_DWORD))(*(_DWORD *)j[5] + 16))(j[5]);
    v50 = j[5];
    sub_2F7B0(a1 + 16, &v50);
    v19 = (void (__fastcall ***)(_DWORD, int))j[5];
    if ( v19 != nullptr )
      (**v19)(j[5], 1);
    j[5] = 0;
    sub_30C80(a1 + 124, j + 4);
    sub_30EB8(a1 + 133, j + 4);
  }
  for ( k = *(_DWORD **)sub_30C2C(v57, &v52); k != (_DWORD *)*sub_1DE18(v57, &v52); k = sub_16A30(k) )
  {
    v42 = k[4];
    if ( v42 <= 4 )
    {
      if ( operator new(0xD0u) == nullptr )
      {
LABEL_27:
        v22 = 0;
        goto LABEL_28;
      }
      v22 = sub_466D0();
    }
    else
    {
      v21 = operator new(0xDCu);
      if ( v21 == nullptr )
        goto LABEL_27;
      v22 = sub_55728(v21, 0, 0, 1, 0, 0);
    }
LABEL_28:
    *(_DWORD *)sub_1DD10(a1 + 124, &v42) = v22;
    if ( *(_DWORD *)sub_1DD10(a1 + 124, &v42) == 0
      || (v23 = sub_1E48C((int)(a1 + 124), &v53, (int)&v42),
          (*(unsigned __int8 (__fastcall **)(_DWORD, _DWORD *, int, int))(**(_DWORD **)(*v23 + 20) + 48))(
            *(_DWORD *)(*v23 + 20),
            a1,
            3,
            v42) == 0) )
    {
      sub_2F89C(v57);
      goto LABEL_32;
    }
    v24 = sub_1E48C((int)(a1 + 124), v54, (int)&v42);
    (*(void (__fastcall **)(_DWORD, void *))(**(_DWORD **)(*v24 + 20) + 44))(*(_DWORD *)(*v24 + 20), v12);
    ReadFile(v12, &v44, 4u, &NumberOfBytesRead, nullptr);
    v25 = (_DWORD *)sub_2FA58(a1 + 133, &v42);
    *v25 = v44;
    ReadFile(v12, &v44, 4u, &NumberOfBytesRead, nullptr);
    v26 = sub_2FA58(a1 + 133, &v42);
    *(_DWORD *)(v26 + 4) = v44;
  }
  v51 = 1;
  v27 = (_DWORD *)sub_2F944(a1 + 130, &v51);
  (*(void (__fastcall **)(_DWORD, void *))(*(_DWORD *)*v27 + 4))(*v27, v12);
  for ( m = 2; m <= 4; ++m )
  {
    v28 = *sub_1DE18(a1 + 124, v54);
    v29 = *sub_1E48C((int)(a1 + 124), &v53, (int)&m) != v28;
    v30 = a1 + 130;
    if ( v29 )
    {
      v32 = (_DWORD *)sub_2F944(v30, &m);
      (*(void (__fastcall **)(_DWORD, void *))(*(_DWORD *)*v32 + 4))(*v32, v12);
    }
    else
    {
      sub_2F944(v30, &m);
      v31 = *(_DWORD *)sub_2F944(a1 + 130, &m);
      if ( v31 != 0 )
        sub_20060(v31, 1);
      *(_DWORD *)sub_2F944(a1 + 130, &m) = 0;
      sub_30DB0(a1 + 130, &m);
    }
  }
  for ( n = *(_DWORD **)sub_2F628(a1 + 127, v54); n != *(_DWORD **)sub_2F640(a1 + 127, &v53); n = sub_16A30(n) )
    (*(void (__fastcall **)(_DWORD, void *, _DWORD *))(*(_DWORD *)n[5] + 4))(n[5], v12, a1);
  sub_346E8((int)(a1 + 152), v12);
  ReadFile(v12, a1 + 136, 6u, &NumberOfBytesRead, nullptr);
  for ( ii = *(_DWORD **)sub_2FB14(a1 + 138, v54); ii != *(_DWORD **)sub_2FB2C(a1 + 138, &v53); ii = sub_16A30(ii) )
  {
    ReadFile(v12, &v40, 2u, &NumberOfBytesRead, nullptr);
    *((_WORD *)ii + 10) = v40;
  }
  for ( jj = *(_DWORD **)sub_2FB14(a1 + 141, v54); jj != *(_DWORD **)sub_2FB2C(a1 + 141, &v53); jj = sub_16A30(jj) )
  {
    ReadFile(v12, &v40, 2u, &NumberOfBytesRead, nullptr);
    *((_WORD *)jj + 10) = v40;
  }
  CloseHandle(v12);
  sub_3958C(a1 + 16, sub_34FF8);
  v36 = a1[149];
  if ( v36 != 0 )
  {
    (*(void (__fastcall **)(_DWORD))(*(_DWORD *)v36 + 12))(a1[149]);
    v37 = (void (__fastcall ***)(_DWORD, int))a1[149];
    if ( v37 != nullptr )
      (**v37)(a1[149], 1);
    a1[149] = 0;
  }
  sub_2F89C(v57);
  sub_12EA0((int)lpFileName);
  sub_12EA0((int)v56);
  return 1;
}

