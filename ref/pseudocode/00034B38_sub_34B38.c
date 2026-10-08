/* ea:   0x00034B38 */
/* name: sub_34B38 */
/* size: 180 */
void *__fastcall sub_34B38(_DWORD *a1, int a2, int a3)
{
  void *result; // r0
  void *v7; // r3
  int (__fastcall ***v8)(_DWORD, int); // r3

  result = operator new(0x50u);
  if ( result != nullptr )
  {
    result = (void *)sub_4F738(result, a1, a2, a3);
    v7 = result;
  }
  else
  {
    v7 = nullptr;
  }
  a1[148] = v7;
  if ( v7 != nullptr )
  {
    result = (void *)(*(int (__fastcall **)(void *, _DWORD, _DWORD))(*(_DWORD *)v7 + 4))(v7, a1[1], a1[2]);
    if ( (_BYTE)result == 0 )
    {
      result = (void *)(*(int (__fastcall **)(_DWORD))(*(_DWORD *)a1[148] + 12))(a1[148]);
      v8 = (int (__fastcall ***)(_DWORD, int))a1[148];
      if ( v8 != nullptr )
        result = (void *)(**v8)(a1[148], 1);
      a1[148] = 0;
    }
  }
  return result;
}

