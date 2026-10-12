/* ea:   0x00065144 */
/* name: sub_65144 */
/* size: 116 */
void (__fastcall ***__fastcall sub_65144(int a1))(_DWORD, int)
{
  void *v2; // r0
  void (__fastcall ***v3)(_DWORD, int); // r4

  v2 = operator new(0x38u);
  if ( v2 != nullptr )
    v3 = (void (__fastcall ***)(_DWORD, int))sub_6683C(v2, a1, 10);
  else
    v3 = nullptr;
  if ( (unsigned __int8)sub_65204(v3) != 0 )
    return v3;
  if ( v3 != nullptr )
    (**v3)(v3, 1);
  return nullptr;
}

