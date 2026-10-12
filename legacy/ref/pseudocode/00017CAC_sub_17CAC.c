/* ea:   0x00017CAC */
/* name: sub_17CAC */
/* size: 232 */
_DWORD *__fastcall sub_17CAC(int a1, int a2)
{
  int v4; // r7
  int v5; // r0
  _DWORD *result; // r0
  _DWORD *v7; // r4
  int *v8; // r6
  int v9; // r0
  _DWORD v10[2]; // [sp+0h] [bp-28h] BYREF
  char v11; // [sp+8h] [bp-20h] BYREF

  v4 = sub_17FD8(a2);
  v5 = sub_180F4(a1, v4);
  if ( v5 != 0 )
    return (_DWORD *)sub_59720(*(_DWORD *)(a1 + 20), v5 + 8);
  result = (_DWORD *)(*(int (__fastcall **)(_DWORD, int, _DWORD, _DWORD))(**(_DWORD **)(a1 + 16) + 8))(
                       *(_DWORD *)(a1 + 16),
                       a2,
                       0,
                       0);
  v7 = result;
  if ( result != nullptr )
  {
    result = operator new(0x7Cu);
    v8 = result;
    if ( result != nullptr )
    {
      result[1] = 0;
      sub_58D18(result + 2);
      *v8 = v4;
      v8[1] = (int)v7;
      v9 = (*(int (__fastcall **)(_DWORD *))(*v7 + 12))(v7);
      sub_58DE0(v8 + 2, v9);
      v10[0] = v4;
      v10[1] = v8;
      sub_17D94(a1, &v11, v10);
      return (_DWORD *)sub_59720(*(_DWORD *)(a1 + 20), v8 + 2);
    }
  }
  return result;
}

