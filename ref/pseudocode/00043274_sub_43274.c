/* ea:   0x00043274 */
/* name: sub_43274 */
/* size: 264 */
int __fastcall sub_43274(_DWORD *a1, int a2, int a3, int a4, int a5)
{
  int result; // r0
  int v9; // r9
  int v10; // r2
  int v11; // r5
  int (*v12)(); // r8
  int v13; // r0
  int v14; // r1
  int v15; // r1
  int (**v16)(); // r0
  _DWORD v17[4]; // [sp+Ch] [bp-40h] BYREF
  _DWORD v18[12]; // [sp+1Ch] [bp-30h] BYREF

  result = sub_4337C(a1, a4, a5);
  if ( (_BYTE)result != 0 )
  {
    v9 = a1[1];
    if ( v9 == 0 )
      __debugbreak();
    v10 = *(_DWORD *)(a1[1] + 4);
    if ( v9 == 0 )
      __debugbreak();
    v11 = *(_DWORD *)(v9 + 8);
    v12 = nullptr;
    v13 = a1[3];
    v14 = a1[4];
    v17[1] = *(_DWORD *)(a1[1] + 4);
    v17[0] = 0;
    v17[2] = 0;
    v17[3] = v11;
    v18[0] = v13 + a4;
    v18[1] = v13 + v10 + a4;
    v18[2] = v14 + a5;
    v18[3] = v14 + v11 + a5;
    if ( v9 == 0 )
      __debugbreak();
    v15 = *(_DWORD *)(v9 + 12);
    if ( v15 == 16 )
    {
      v16 = off_7BB78;
    }
    else
    {
      if ( v15 != 8 )
        return (*(int (__fastcall **)(int, _DWORD *, int, _DWORD *, int (*)(), int *, int))(*(_DWORD *)a2 + 20))(
                 a2,
                 v18,
                 v9,
                 v17,
                 v12,
                 &dword_81230,
                 1);
      v16 = off_7BBB8;
    }
    v12 = v16[8];
    return (*(int (__fastcall **)(int, _DWORD *, int, _DWORD *, int (*)(), int *, int))(*(_DWORD *)a2 + 20))(
             a2,
             v18,
             v9,
             v17,
             v12,
             &dword_81230,
             1);
  }
  return result;
}

