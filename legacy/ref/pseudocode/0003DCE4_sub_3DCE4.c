/* ea:   0x0003DCE4 */
/* name: sub_3DCE4 */
/* size: 272 */
int __fastcall sub_3DCE4(int a1, int a2)
{
  char v4; // r6
  int result; // r0
  int v6; // r5
  void (__fastcall ***v7)(_DWORD, int); // r3

  v4 = *(_BYTE *)(a1 + 33);
  *(_BYTE *)(a1 + 33) = 0;
  result = (int)operator new(0x2ECu);
  if ( result != 0 )
  {
    result = sub_1EECC(result, *(_DWORD *)(a1 + 20), *(_DWORD *)(a1 + 24));
    v6 = result;
    if ( result != 0 )
    {
      if ( (*(unsigned __int8 (__fastcall **)(int, int, int, _DWORD))(*(_DWORD *)result + 32))(
             result,
             a2,
             a1,
             *(_DWORD *)(a1 + 4)) != 0 )
      {
        if ( *(_DWORD *)(a1 + 8) != 0 )
        {
          (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 4) + 24))(*(_DWORD *)(a1 + 4));
          (*(void (__fastcall **)(_DWORD))(**(_DWORD **)(a1 + 8) + 12))(*(_DWORD *)(a1 + 8));
          v7 = *(void (__fastcall ****)(_DWORD, int))(a1 + 8);
          if ( v7 != nullptr )
            (**v7)(*(_DWORD *)(a1 + 8), 1);
          *(_DWORD *)(a1 + 8) = 0;
        }
        *(_DWORD *)(a1 + 8) = v6;
        *(_DWORD *)a1 = 12;
        *(_BYTE *)(a1 + 33) = v4;
        return 12;
      }
      else
      {
        (*(void (__fastcall **)(int))(*(_DWORD *)v6 + 12))(v6);
        return (**(int (__fastcall ***)(int, int))v6)(v6, 1);
      }
    }
  }
  return result;
}

