/* ea:   0x0005F19C */
/* name: sub_5F19C */
/* size: 764 */
int __fastcall sub_5F19C(int a1, LPCWSTR *a2)
{
  LPCWSTR v3; // r1
  int v5; // r6
  _DWORD *v6; // r10
  _BYTE *v7; // r3
  __int16 v8; // t1
  unsigned int v9; // r4
  _DWORD *i; // r7
  _BYTE *v11; // r3
  __int16 v12; // t1
  unsigned int v13; // r5
  int (**v14)(); // [sp+4h] [bp-24Ch] BYREF
  HANDLE hFile; // [sp+8h] [bp-248h]
  DWORD NumberOfBytesWritten; // [sp+Ch] [bp-244h] BYREF
  __int16 Buffer; // [sp+10h] [bp-240h] BYREF
  LPCWSTR *v18; // [sp+14h] [bp-23Ch]
  DWORD v19; // [sp+18h] [bp-238h] BYREF
  _BYTE v20[4]; // [sp+1Ch] [bp-234h] BYREF
  _BYTE v21[4]; // [sp+20h] [bp-230h] BYREF
  _BYTE v22[6]; // [sp+24h] [bp-22Ch] BYREF
  _BYTE v23[2]; // [sp+2Ah] [bp-226h] BYREF
  wchar_t v24[274]; // [sp+2Ch] [bp-224h] BYREF

  v18 = a2;
  hFile = (HANDLE)-1;
  v3 = *a2;
  v14 = off_7BC04;
  if ( (unsigned __int8)sub_63224(&v14, v3, 1, 1) == 0 )
  {
    v14 = off_7BC04;
    sub_611DC(&v14);
    return 0;
  }
  Buffer = -257;
  if ( ((unsigned __int8 (__fastcall *)(int (***)()))v14[9])(&v14) == 0 )
    __debugbreak();
  if ( WriteFile(hFile, &Buffer, 2u, &NumberOfBytesWritten, nullptr) && NumberOfBytesWritten == 0 )
    goto LABEL_7;
  v5 = a1 + 4;
  v6 = *(_DWORD **)sub_6038C(a1 + 4, &v19);
LABEL_9:
  if ( v6 == *(_DWORD **)sub_603A4(v5, v20) )
  {
    sub_6118C(&v14);
    return 1;
  }
  else
  {
    swprintf(v24, aS_6, v6[5]);
    v7 = v23;
    do
    {
      v8 = *((_WORD *)v7 + 1);
      v7 += 2;
    }
    while ( v8 != 0 );
    v9 = (unsigned int)(v7 - (_BYTE *)v24) >> 1;
    if ( ((unsigned __int8 (__fastcall *)(int (***)()))v14[9])(&v14) == 0 )
      __debugbreak();
    if ( WriteFile(hFile, v24, 2 * v9, &NumberOfBytesWritten, nullptr) && NumberOfBytesWritten == 0 )
    {
LABEL_7:
      DeleteFileW(*v18);
      v14 = off_7BC04;
      sub_611DC(&v14);
      return 0;
    }
    for ( i = *(_DWORD **)sub_60350(v6 + 8, v21); ; i = sub_16A30(i) )
    {
      if ( i == *(_DWORD **)sub_60368(v6 + 8, v22) )
      {
        v6 = sub_16A30(v6);
        goto LABEL_9;
      }
      swprintf(v24, aSS, i[5], i[8]);
      v11 = v23;
      do
      {
        v12 = *((_WORD *)v11 + 1);
        v11 += 2;
      }
      while ( v12 != 0 );
      v13 = (unsigned int)(v11 - (_BYTE *)v24) >> 1;
      if ( ((unsigned __int8 (__fastcall *)(int (***)()))v14[9])(&v14) == 0 )
        __debugbreak();
      if ( WriteFile(hFile, v24, 2 * v13, &v19, nullptr) && v19 == 0 )
        break;
    }
    DeleteFileW(*v18);
    sub_6118C(&v14);
    return 0;
  }
}

