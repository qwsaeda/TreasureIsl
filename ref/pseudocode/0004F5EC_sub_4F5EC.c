/* ea:   0x0004F5EC */
/* name: sub_4F5EC */
/* size: 332 */
int __fastcall sub_4F5EC(_DWORD *a1, int a2, int a3, int a4, int a5)
{
  int result; // r0
  int v9; // r10
  int v10; // r2
  signed int v11; // r4
  int v12; // r3
  int v13; // r5
  int (*v14)(); // r9
  int v15; // r0
  bool v16; // cc
  int v17; // r0
  int v18; // r0
  int v19; // r1
  int v20; // r1
  int (**v21)(); // r0
  _DWORD v22[4]; // [sp+Ch] [bp-44h] BYREF
  _DWORD v23[13]; // [sp+1Ch] [bp-34h] BYREF

  result = sub_4337C(a1, a4, a5);
  if ( (_BYTE)result != 0 )
  {
    v9 = a1[1];
    if ( v9 == 0 )
      __debugbreak();
    v10 = (int)&rt_udiv;
    v11 = a1[8];
    v12 = *(_DWORD *)(a1[1] + 4) / (unsigned int)v11;
    if ( v9 == 0 )
      __debugbreak();
    v13 = *(_DWORD *)(v9 + 8);
    v14 = nullptr;
    v15 = a1[11];
    v22[2] = 0;
    v16 = v15 < v11;
    v22[3] = v13;
    if ( v15 >= v11 )
      a1[11] = 0;
    else
      v10 = v15 * v12;
    v17 = a1[11];
    if ( !v16 )
      v10 = 0;
    v22[0] = v10;
    a1[11] = v17 + 1;
    v22[1] = v10 + v12;
    v18 = a1[3];
    v19 = a1[4];
    v23[0] = v18 + a4;
    v23[1] = v18 + v12 + a4;
    v23[2] = v19 + a5;
    v23[3] = v19 + v13 + a5;
    if ( v9 == 0 )
      __debugbreak();
    v20 = *(_DWORD *)(v9 + 12);
    if ( v20 == 16 )
    {
      v21 = off_7BB78;
    }
    else
    {
      if ( v20 != 8 )
        return (*(int (__fastcall **)(int, _DWORD *, int, _DWORD *, int (*)(), int *, int))(*(_DWORD *)a2 + 20))(
                 a2,
                 v23,
                 v9,
                 v22,
                 v14,
                 &dword_81230,
                 1);
      v21 = off_7BBB8;
    }
    v14 = v21[8];
    return (*(int (__fastcall **)(int, _DWORD *, int, _DWORD *, int (*)(), int *, int))(*(_DWORD *)a2 + 20))(
             a2,
             v23,
             v9,
             v22,
             v14,
             &dword_81230,
             1);
  }
  return result;
}

