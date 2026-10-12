/* ea:   0x00042FE4 */
/* name: sub_42FE4 */
/* size: 200 */
_DWORD *__fastcall sub_42FE4(int a1)
{
  _DWORD *result; // r0
  _DWORD *v3; // r6
  int v4; // r3
  int v5; // r4
  int (*v6)(); // r2
  int v7; // r0
  int (**v8)(); // r0
  _DWORD v9[4]; // [sp+Ch] [bp-34h] BYREF
  _DWORD v10[9]; // [sp+1Ch] [bp-24h] BYREF

  result = *(_DWORD **)(a1 + 12);
  v3 = result;
  if ( result != nullptr )
  {
    v4 = result[1];
    v5 = result[2];
    v10[2] = 280;
    v9[1] = v4;
    v9[3] = v5;
    v10[0] = (176 - v4) / 2;
    v6 = nullptr;
    v10[1] = v10[0] + v4;
    v9[0] = 0;
    v10[3] = v5 + 280;
    v9[2] = 0;
    v7 = result[3];
    if ( v7 == 16 )
    {
      v8 = off_7BB78;
    }
    else
    {
      if ( v7 != 8 )
        return (_DWORD *)(*(int (__fastcall **)(_DWORD, _DWORD *, _DWORD *, _DWORD *, int (*)(), int *, int))(**(_DWORD **)(a1 + 4) + 20))(
                           *(_DWORD *)(a1 + 4),
                           v10,
                           v3,
                           v9,
                           v6,
                           &dword_81230,
                           1);
      v8 = off_7BBB8;
    }
    v6 = v8[8];
    return (_DWORD *)(*(int (__fastcall **)(_DWORD, _DWORD *, _DWORD *, _DWORD *, int (*)(), int *, int))(**(_DWORD **)(a1 + 4) + 20))(
                       *(_DWORD *)(a1 + 4),
                       v10,
                       v3,
                       v9,
                       v6,
                       &dword_81230,
                       1);
  }
  return result;
}

