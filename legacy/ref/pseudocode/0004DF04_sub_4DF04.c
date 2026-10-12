/* ea:   0x0004DF04 */
/* name: sub_4DF04 */
/* size: 20 */
void __fastcall sub_4DF04(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

