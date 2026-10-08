/* ea:   0x0004290C */
/* name: sub_4290C */
/* size: 296 */
_DWORD *__fastcall sub_4290C(_DWORD *a1, int a2, int a3, int a4, int a5)
{
  int v7; // r0
  int v8; // r0
  int v9; // r0
  int v10; // r5
  void *v11; // r4
  _WORD *v13; // [sp+0h] [bp-70h] BYREF
  void *v14[4]; // [sp+8h] [bp-68h] BYREF
  int (__fastcall **v15)(void *); // [sp+18h] [bp-58h] BYREF
  char v16; // [sp+1Ch] [bp-54h]
  int v17; // [sp+20h] [bp-50h]
  char v18; // [sp+24h] [bp-4Ch]
  _DWORD v19[16]; // [sp+30h] [bp-40h] BYREF

  *a1 = a4;
  a1[2] = a5;
  a1[1] = a3;
  a1[3] = 0;
  a1[4] = 0;
  a1[5] = 0;
  sub_1E61C((int)v19);
  v18 = 0;
  v15 = &off_6C938;
  v16 = 0;
  sub_1333C(v14);
  if ( (unsigned __int8)sub_63BE8(v19, a2) != 0 )
  {
    v7 = sub_12F04(aDemoPcx);
    sub_1318C(v14, (char *)aDemoPcx, (int)&aDemoPcx[v7]);
    v13 = v14[0];
    v8 = sub_1E5CC(&v13);
    v9 = sub_1E84C((int)v19, v8);
    v10 = v9;
    if ( v9 == 0 )
      __debugbreak();
    v11 = operator new(*(_DWORD *)(v9 + 4));
    sub_63CE8((int)v19, v11);
    v17 = 16;
    v16 = 1;
    a1[3] = sub_645B4(&v15, v11, *(_DWORD *)(v10 + 4));
    if ( v11 != nullptr )
      operator delete(v11);
  }
  sub_12EA0((int)v14);
  v15 = &off_6C910;
  sub_1E798(v19);
  return a1;
}

