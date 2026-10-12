/* ea:   0x00016570 */
/* name: sub_16570 */
/* size: 44 */
unsigned int __fastcall sub_16570(unsigned int *a1, unsigned int a2)
{
  unsigned int v3; // r0
  unsigned int result; // r0

  v3 = *a1;
  if ( a2 < v3 )
  {
    result = v3 - a2;
    *a1 = result;
  }
  else
  {
    *a1 = 0;
    return 0;
  }
  return result;
}

