/* ea:   0x000637C8 */
/* name: sub_637C8 */
/* size: 128 */
void (__fastcall ***__fastcall sub_637C8(int a1))(_DWORD, int)
{
  void (__fastcall ***v2)(_DWORD, int); // r0
  void (__fastcall ***v3)(_DWORD, int); // r5
  bool v4; // zf
  void (__fastcall ***result)(_DWORD, int); // r0

  v2 = (void (__fastcall ***)(_DWORD, int))operator new(0x14u);
  v3 = v2;
  if ( v2 == nullptr )
    return nullptr;
  v2[2] = nullptr;
  *v2 = (void (__fastcall **)(_DWORD, int))&off_7BCCC;
  v2[3] = nullptr;
  v2[4] = nullptr;
  *((_BYTE *)v2 + 4) = 0;
  v4 = ((unsigned __int8 (__fastcall *)(void *, int))(*v2)[1])(v2, a1) == 0;
  result = v3;
  if ( v4 )
  {
    (**v3)(v3, 1);
    return nullptr;
  }
  return result;
}

