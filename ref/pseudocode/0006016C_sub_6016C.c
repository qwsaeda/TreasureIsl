/* ea:   0x0006016C */
/* name: sub_6016C */
/* size: 208 */
int __fastcall sub_6016C(_DWORD *a1, int *a2)
{
  int v4; // r6
  int v5; // r4
  _BYTE v7[8]; // [sp+0h] [bp-44h] BYREF
  void *v8[4]; // [sp+8h] [bp-3Ch] BYREF
  int v9; // [sp+18h] [bp-2Ch] BYREF
  _DWORD v10[3]; // [sp+1Ch] [bp-28h] BYREF
  char v11; // [sp+28h] [bp-1Ch] BYREF

  v4 = sub_60550();
  if ( v4 != *a1 && (unsigned int)*a2 >= *(_DWORD *)(v4 + 16) )
    return v4 + 32;
  sub_6023C(v8);
  v9 = *a2;
  sub_132FC(v10, a2 + 1);
  sub_602E8(&v11, v8);
  v5 = *(_DWORD *)sub_60D94(a1, v7, v4, &v9);
  sub_19910((int)&v9);
  sub_1994C(v8);
  sub_60504((int)v8, v8[0]);
  return v5 + 32;
}

