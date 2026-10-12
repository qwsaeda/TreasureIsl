/* ea:   0x00019604 */
/* name: sub_19604 */
/* size: 44 */
int __fastcall sub_19604(int *a1)
{
  int result; // r0

  result = *a1;
  if ( result != 0 )
    return (*(int (__fastcall **)(int, int))(*(_DWORD *)result + 4))(result, 1);
  return result;
}

