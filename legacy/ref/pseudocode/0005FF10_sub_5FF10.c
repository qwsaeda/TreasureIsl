/* ea:   0x0005FF10 */
/* name: sub_5FF10 */
/* size: 352 */
int __fastcall sub_5FF10(int a1, int a2, _WORD **a3)
{
  _WORD *v5; // r4
  int v6; // r0
  _DWORD *v7; // r0
  int v8; // r0
  int v9; // r4
  char v11; // [sp+2h] [bp-72h] BYREF
  _BYTE v12[5]; // [sp+3h] [bp-71h] BYREF
  _DWORD v13[2]; // [sp+8h] [bp-6Ch] BYREF
  _BYTE v14[8]; // [sp+10h] [bp-64h] BYREF
  void *v15[4]; // [sp+18h] [bp-5Ch] BYREF
  _DWORD v16[4]; // [sp+28h] [bp-4Ch] BYREF
  int v17; // [sp+38h] [bp-3Ch] BYREF
  void *v18[3]; // [sp+3Ch] [bp-38h] BYREF
  _BYTE v19[4]; // [sp+48h] [bp-2Ch] BYREF
  _BYTE v20[12]; // [sp+4Ch] [bp-28h] BYREF
  _BYTE v21[28]; // [sp+58h] [bp-1Ch] BYREF

  v5 = *a3;
  sub_3E0A4(v15);
  v6 = sub_12F04(v5);
  sub_183A4(v15, v5, &v5[v6]);
  v17 = sub_19A4C((_WORD *)v15[0]);
  sub_132FC(v18, v15);
  sub_12F48(v15);
  v7 = (_DWORD *)sub_6157C(a1 + 4, v13, &v17);
  if ( *v7 == *(_DWORD *)(a1 + 4) )
  {
    sub_602FC(v16, v12, &v11);
    sub_602C4(v19, &v17);
    sub_602E8(v21, v16);
    v8 = sub_613BC(a1 + 4, v13, v19);
    sub_603D0(v14, v8);
    sub_6014C(v21);
    sub_12EA0((int)v20);
    sub_1994C(v16);
    sub_60378(v16);
    v9 = sub_6016C(a1 + 4, &v17);
    sub_602C4(a2, &v17);
    *(_DWORD *)(a2 + 16) = v9;
    sub_12EA0((int)v18);
  }
  else
  {
    v13[0] = *v7 + 32;
    sub_60260(a2, &v17, v13);
    sub_12F48(v18);
  }
  return a2;
}

