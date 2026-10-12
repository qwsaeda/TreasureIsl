/* ea:   0x0005F55C */
/* name: sub_5F55C */
/* size: 212 */
int __fastcall sub_5F55C(int a1, int a2, int a3, void **a4)
{
  int v7; // r6
  int v8; // r4
  char *v9; // r4
  int v10; // r0
  int v12; // [sp+0h] [bp-3Ch] BYREF
  void *v13[3]; // [sp+4h] [bp-38h] BYREF
  int v14; // [sp+10h] [bp-2Ch] BYREF
  void *v15[10]; // [sp+14h] [bp-28h] BYREF

  v12 = a2;
  sub_1333C(v13);
  v7 = sub_61918(a1 + 4, &v12);
  sub_12F48(v13);
  if ( v7 == *(_DWORD *)(a1 + 4) )
    return 0;
  v14 = a3;
  sub_1333C(v15);
  v8 = sub_61890(v7 + 32, &v14);
  sub_12F48(v15);
  if ( v8 == *(_DWORD *)(v7 + 32) )
    return 0;
  v9 = *(char **)(v8 + 32);
  v10 = sub_12F04(v9);
  sub_1318C(a4, v9, (int)&v9[2 * v10]);
  return 1;
}

