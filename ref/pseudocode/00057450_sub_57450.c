/* ea:   0x00057450 */
/* name: sub_57450 */
/* size: 360 */
_DWORD *__fastcall sub_57450(int a1, _DWORD *a2)
{
  void *v4; // r0
  int v5; // r1
  int v6; // r2
  int v7; // r3
  int v8; // r1
  int v9; // r2
  int v10; // r3
  int v11; // r1
  int v12; // r2
  int v13; // r3
  int v14; // r3
  _DWORD *result; // r0

  v4 = off_811A0;
  if ( off_811A0 != nullptr )
  {
    sub_5850C();
    v4 = off_811A0;
  }
  sub_17C7C((int)v4, 2);
  v5 = a2[1];
  v6 = a2[2];
  v7 = a2[3];
  *(_DWORD *)(a1 + 1052) = *a2;
  *(_DWORD *)(a1 + 1060) = v6;
  *(_DWORD *)(a1 + 1056) = v5;
  *(_DWORD *)(a1 + 1064) = v7;
  v8 = a2[5];
  v9 = a2[6];
  v10 = a2[7];
  *(_DWORD *)(a1 + 1068) = a2[4];
  *(_DWORD *)(a1 + 1076) = v9;
  *(_DWORD *)(a1 + 1072) = v8;
  *(_DWORD *)(a1 + 1080) = v10;
  v11 = a2[9];
  v12 = a2[10];
  v13 = a2[11];
  *(_DWORD *)(a1 + 1084) = a2[8];
  *(_DWORD *)(a1 + 1092) = v12;
  *(_DWORD *)(a1 + 1088) = v11;
  *(_DWORD *)(a1 + 1096) = v13;
  LOWORD(v11) = *(_WORD *)(a1 + 1090);
  *(_DWORD *)(a1 + 1100) = a1 + 1052;
  v14 = 10 * (unsigned __int16)v11;
  LOWORD(v11) = *(_WORD *)(a1 + 1084);
  *(_DWORD *)(a1 + 1124) = v14;
  *(_DWORD *)(a1 + 1128) = 10 * (unsigned __int16)v11;
  result = *(_DWORD **)(a1 + 1052);
  if ( result == (_DWORD *)1 )
  {
    sub_445F4(a1, 100)[5] = 0;
    result = sub_445F4(a1, 300);
    result[5] = 0;
  }
  else if ( result == (_DWORD *)2 )
  {
    sub_445F4(a1, 100)[5] = 0;
    result = sub_445F4(a1, 200);
    result[5] = 0;
  }
  else if ( result == (_DWORD *)3 )
  {
    sub_445F4(a1, 200)[5] = 0;
    result = sub_445F4(a1, 300);
    result[5] = 0;
  }
  return result;
}

