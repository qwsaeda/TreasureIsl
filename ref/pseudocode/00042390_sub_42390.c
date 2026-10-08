/* ea:   0x00042390 */
/* name: sub_42390 */
/* size: 424 */
int __fastcall sub_42390(int a1, int a2, _DWORD *a3)
{
  int v5; // r0
  int v6; // r7
  void **v7; // r6
  int v8; // r0
  int v9; // r0
  int v10; // r5
  void *v11; // r0
  void *v12; // r4
  _WORD *v14; // [sp+4h] [bp-98h] BYREF
  void *v15[4]; // [sp+8h] [bp-94h] BYREF
  int (**v16)(); // [sp+18h] [bp-84h] BYREF
  void *v17[5]; // [sp+1Ch] [bp-80h] BYREF
  int (__fastcall **v18)(void *); // [sp+30h] [bp-6Ch] BYREF
  char v19; // [sp+34h] [bp-68h]
  int v20; // [sp+38h] [bp-64h]
  _DWORD v21[21]; // [sp+48h] [bp-54h] BYREF

  sub_1E61C((int)v21);
  sub_1DCEC((int)&v18);
  v18 = &off_6C938;
  sub_19490((int)v17);
  v17[3] = nullptr;
  v16 = &off_6C90C;
  sub_1333C(v15);
  if ( (unsigned __int8)sub_63BE8(v21, a2) != 0 )
  {
    v5 = sub_12F04(aLogoPcx);
    sub_1318C(v15, (char *)aLogoPcx, (int)&aLogoPcx[v5]);
    sub_1DDA4((int)a3, 1u);
    v6 = 0;
    v7 = v15;
    while ( 1 )
    {
      v14 = *v7;
      v8 = sub_1E5CC(&v14);
      v9 = sub_1E84C((int)v21, v8);
      v10 = v9;
      if ( v9 == 0 )
        __debugbreak();
      v11 = operator new(*(_DWORD *)(v9 + 4));
      v12 = v11;
      if ( v11 == nullptr )
        break;
      sub_63CE8((int)v21, v11);
      v20 = 16;
      v19 = 1;
      *(_DWORD *)(v6 + *a3) = sub_645B4(&v18, v12, *(_DWORD *)(v10 + 4));
      operator delete(v12);
      if ( *(_DWORD *)(v6 + *a3) == 0 )
        break;
      v6 += 4;
      v7 += 3;
      if ( v6 >= 4 )
        goto LABEL_8;
    }
    sub_12EA0((int)v15);
    v16 = &off_6C90C;
    sub_194B4(v17);
    v18 = &off_6C910;
    sub_1E798(v21);
    return 0;
  }
  else
  {
LABEL_8:
    sub_12EA0((int)v15);
    sub_199D8(&v16);
    sub_1EA14(&v18);
    sub_1E798(v21);
    return 1;
  }
}

