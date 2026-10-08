/* ea:   0x00012F04 */
/* name: sub_12F04 */
/* size: 68 */
int __fastcall sub_12F04(_WORD *a1)
{
  int v2; // r4

  v2 = 0;
  while ( *a1 != 0 )
  {
    ++v2;
    ++a1;
  }
  return v2;
}

