/* ea:   0x0001F20C */
/* name: sub_1F20C */
/* size: 1452 */
int __fastcall sub_1F20C(int a1, int a2)
{
  wchar_t *v4; // r4
  int v5; // r0
  __int16 **v6; // r0
  HANDLE FileW; // r0
  void *v8; // r8
  _DWORD *i; // r4
  _DWORD *j; // r5
  _DWORD *k; // r5
  _DWORD *m; // r5
  _DWORD *n; // r5
  _DWORD *ii; // r5
  __int16 v16; // [sp+Ch] [bp-78h] BYREF
  DWORD NumberOfBytesWritten; // [sp+10h] [bp-74h] BYREF
  int v18; // [sp+14h] [bp-70h] BYREF
  int Buffer; // [sp+18h] [bp-6Ch] BYREF
  int v20; // [sp+1Ch] [bp-68h] BYREF
  int v21; // [sp+20h] [bp-64h] BYREF
  int v22; // [sp+24h] [bp-60h] BYREF
  int v23; // [sp+28h] [bp-5Ch] BYREF
  int v24; // [sp+2Ch] [bp-58h] BYREF
  int v25; // [sp+30h] [bp-54h] BYREF
  LPCWSTR lpFileName[4]; // [sp+34h] [bp-50h] BYREF
  void *v27[4]; // [sp+44h] [bp-40h] BYREF
  void *v28[12]; // [sp+54h] [bp-30h] BYREF

  sub_1333C(v27);
  sub_1333C(lpFileName);
  sub_1309C((int)v27);
  switch ( a2 )
  {
    case 1:
      v4 = aDataGl1Sav;
      break;
    case 2:
      v4 = aDataGl2Sav;
      break;
    case 3:
      v4 = aDataGl3Sav;
      break;
    case 4:
      v4 = aDataGl4Sav;
      break;
    default:
      v4 = aDataGl0Sav;
      break;
  }
  v5 = sub_12F04(v4);
  sub_1318C((void **)lpFileName, (char *)v4, (int)&v4[v5]);
  v6 = (__int16 **)sub_37E2C(v28, v27, lpFileName);
  if ( v6 != (__int16 **)lpFileName )
    sub_17890((int)lpFileName, *v6, v6[1]);
  sub_12F48(v28);
  FileW = CreateFileW(lpFileName[0], 0x40000000u, 0, nullptr, 2u, 0x80u, nullptr);
  v8 = FileW;
  if ( FileW == (HANDLE)-1 )
  {
    sub_12EA0((int)lpFileName);
    sub_12EA0((int)v27);
    return 0;
  }
  else
  {
    SetFilePointer(FileW, 2, nullptr, 0);
    Buffer = 19810929;
    WriteFile(v8, &Buffer, 4u, &NumberOfBytesWritten, nullptr);
    v20 = **(_DWORD **)(a1 + 264);
    WriteFile(v8, &v20, 4u, &NumberOfBytesWritten, nullptr);
    WriteFile(v8, &dword_7FB70, 4u, &NumberOfBytesWritten, nullptr);
    WriteFile(v8, &dword_7FB6C, 4u, &NumberOfBytesWritten, nullptr);
    (**(void (__fastcall ***)(int, void *))(a1 + 144))(a1 + 144, v8);
    (**(void (__fastcall ***)(int, void *))(a1 + 72))(a1 + 72, v8);
    WriteFile(v8, (LPCVOID)(a1 + 724), 4u, &NumberOfBytesWritten, nullptr);
    (*(void (__fastcall **)(int, void *))(*(_DWORD *)(a1 + 288) + 40))(a1 + 288, v8);
    v21 = 1;
    v18 = *(_DWORD *)sub_2FA58(a1 + 532, &v21);
    WriteFile(v8, &v18, 4u, &NumberOfBytesWritten, nullptr);
    v22 = 1;
    v18 = *(_DWORD *)(sub_2FA58(a1 + 532, &v22) + 4);
    WriteFile(v8, &v18, 4u, &NumberOfBytesWritten, nullptr);
    v23 = *(_DWORD *)(a1 + 500);
    WriteFile(v8, &v23, 4u, &NumberOfBytesWritten, nullptr);
    for ( i = *(_DWORD **)sub_30C2C(a1 + 496, &v25); i != (_DWORD *)*sub_1DE18((_DWORD *)(a1 + 496), &v25); i = sub_16A30(i) )
    {
      v24 = i[4];
      WriteFile(v8, &v24, 4u, &NumberOfBytesWritten, nullptr);
    }
    for ( j = *(_DWORD **)sub_30C2C(a1 + 496, &v25); j != (_DWORD *)*sub_1DE18((_DWORD *)(a1 + 496), &v25); j = sub_16A30(j) )
    {
      (*(void (__fastcall **)(_DWORD, void *))(*(_DWORD *)j[5] + 40))(j[5], v8);
      v18 = *(_DWORD *)sub_2FA58(a1 + 532, j + 4);
      WriteFile(v8, &v18, 4u, &NumberOfBytesWritten, nullptr);
      v18 = *(_DWORD *)(sub_2FA58(a1 + 532, j + 4) + 4);
      WriteFile(v8, &v18, 4u, &NumberOfBytesWritten, nullptr);
    }
    for ( k = *(_DWORD **)sub_30D88(a1 + 520, &v25); k != *(_DWORD **)sub_30DA0(a1 + 520, &v25); k = sub_16A30(k) )
      (**(void (__fastcall ***)(_DWORD, void *))k[5])(k[5], v8);
    for ( m = (_DWORD *)*sub_1DDF0(a1 + 508, &v25); m != (_DWORD *)*sub_1DE08((_DWORD *)(a1 + 508), &v25); m = sub_16A30(m) )
      (**(void (__fastcall ***)(_DWORD, void *))m[5])(m[5], v8);
    sub_34588(a1 + 608, v8);
    WriteFile(v8, (LPCVOID)(a1 + 544), 6u, &NumberOfBytesWritten, nullptr);
    for ( n = *(_DWORD **)sub_310BC(a1 + 552, &v25); n != *(_DWORD **)sub_310D4(a1 + 552, &v25); n = sub_16A30(n) )
    {
      v16 = *((_WORD *)n + 10);
      WriteFile(v8, &v16, 2u, &NumberOfBytesWritten, nullptr);
    }
    for ( ii = *(_DWORD **)sub_310BC(a1 + 564, &v25); ii != *(_DWORD **)sub_310D4(a1 + 564, &v25); ii = sub_16A30(ii) )
    {
      v16 = *((_WORD *)ii + 10);
      WriteFile(v8, &v16, 2u, &NumberOfBytesWritten, nullptr);
    }
    CloseHandle(v8);
    sub_12F48((void **)lpFileName);
    sub_12F48(v27);
    return 1;
  }
}

