/* ea:   0x0006221C */
/* name: sub_6221C */
/* size: 124 */
void (__fastcall ***__fastcall sub_6221C(int a1, int a2))(_DWORD, int)
{
  _DWORD *v4; // r0
  void (__fastcall ***v5)(_DWORD, int); // r6
  bool v6; // zf
  void (__fastcall ***result)(_DWORD, int); // r0

  v4 = operator new(0x1Cu);
  v5 = (void (__fastcall ***)(_DWORD, int))v4;
  if ( v4 == nullptr )
    return nullptr;
  v4[2] = 0;
  *v4 = off_7BC2C;
  v6 = (*(unsigned __int8 (__fastcall **)(_DWORD *, int, int))(*v4 + 4))(v4, a1, a2) == 0;
  result = v5;
  if ( v6 )
  {
    (**v5)(v5, 1);
    return nullptr;
  }
  return result;
}

