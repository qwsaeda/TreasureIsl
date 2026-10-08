/* ea:   0x0003DC04 */
/* name: sub_3DC04 */
/* size: 224 */
int __fastcall sub_3DC04(int a1, int a2)
{
  int result; // r0
  int v5; // r4
  void (__fastcall ***v6)(_DWORD, int); // r3

  result = sub_3DDF4();
  v5 = result;
  if ( result != 0 )
  {
    if ( (*(unsigned __int8 (__fastcall **)(int, int, _DWORD))(*(_DWORD *)result + 4))(result, a1, *(_DWORD *)(a1 + 4)) != 0 )
    {
      result = *(_DWORD *)(a1 + 8);
      if ( result != 0 )
      {
        (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 4) + 24))(*(_DWORD *)(a1 + 4));
        (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 8) + 12))(*(_DWORD *)(a1 + 8));
        v6 = *(void (__fastcall ****)(_DWORD, int))(a1 + 8);
        if ( v6 != nullptr )
          (**v6)(*(_DWORD *)(a1 + 8), 1);
        result = 0;
        *(_DWORD *)(a1 + 8) = 0;
      }
      *(_DWORD *)a1 = a2;
      *(_DWORD *)(a1 + 8) = v5;
    }
    else
    {
      (*(void (__fastcall **)(int))(*(_DWORD *)v5 + 12))(v5);
      return (**(int (__fastcall ***)(int, int))v5)(v5, 1);
    }
  }
  return result;
}

