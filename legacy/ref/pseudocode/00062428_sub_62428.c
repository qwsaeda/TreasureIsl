/* ea:   0x00062428 */
/* name: sub_62428 */
/* size: 76 */
_DWORD *__fastcall sub_62428(_DWORD *a1, int *a2)
{
  int v3; // r4
  void (__fastcall ***v4)(_DWORD, int); // r2

  v3 = *a2;
  *a2 = 0;
  v4 = (void (__fastcall ***)(_DWORD, int))*a1;
  if ( v3 != *a1 && v4 != nullptr )
    (**v4)(*a1, 1);
  *a1 = v3;
  return a1;
}

