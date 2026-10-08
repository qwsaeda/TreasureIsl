/* ea:   0x000625E0 */
/* name: sub_625E0 */
/* size: 60 */
int (**__fastcall sub_625E0(_DWORD *a1))()
{
  int (**result)(); // r0
  int (__fastcall ***v3)(_DWORD, int); // r1

  result = off_7BC58;
  *a1 = off_7BC58;
  v3 = (int (__fastcall ***)(_DWORD, int))a1[2];
  if ( v3 != nullptr )
    return (int (**)())(**v3)(v3, 1);
  return result;
}

